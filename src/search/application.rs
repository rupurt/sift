use super::adapters::*;
use super::domain::{
    Bm25Index, Candidate, CandidateList, CorpusRepository, Embedder, Expander, Fuser, FusionPolicy,
    GenerativeModel, LoadedCorpus, PreparedCorpus, QueryEmbeddingCache, QueryExpansionPolicy,
    Reranker, RerankingPolicy, Retriever, RetrieverPolicy, SearchCoverageSnapshot, SearchHit,
    SearchPhase, SearchPlan, SearchProgress, SearchRequest, SearchResponse, StrategyPresetRegistry,
    tokenize,
};
use crate::cache::resolve_compatible_cache_path;
use crate::config::Ignore;
use anyhow::{Result, anyhow};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

pub struct SearchService {
    retrievers: std::collections::HashMap<RetrieverPolicy, Arc<dyn Retriever>>,
    fusers: std::collections::HashMap<FusionPolicy, Box<dyn Fuser>>,
    expanders: std::collections::HashMap<QueryExpansionPolicy, Box<dyn Expander>>,
    rerankers: std::collections::HashMap<RerankingPolicy, Arc<dyn Reranker>>,
}

impl Default for SearchService {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchService {
    pub fn new() -> Self {
        Self {
            retrievers: std::collections::HashMap::new(),
            fusers: std::collections::HashMap::new(),
            expanders: std::collections::HashMap::new(),
            rerankers: std::collections::HashMap::new(),
        }
    }

    pub fn register_retriever(&mut self, retriever: Box<dyn Retriever>) {
        self.retrievers
            .insert(retriever.policy(), Arc::from(retriever));
    }

    pub fn register_fuser(&mut self, policy: FusionPolicy, fuser: Box<dyn Fuser>) {
        self.fusers.insert(policy, fuser);
    }

    pub fn register_expander(&mut self, policy: QueryExpansionPolicy, expander: Box<dyn Expander>) {
        self.expanders.insert(policy, expander);
    }

    pub fn register_reranker(&mut self, policy: RerankingPolicy, reranker: Box<dyn Reranker>) {
        self.rerankers.insert(policy, Arc::from(reranker));
    }

    pub fn register_reranker_arc(&mut self, policy: RerankingPolicy, reranker: Arc<dyn Reranker>) {
        self.rerankers.insert(policy, reranker);
    }

    pub fn execute(
        &self,
        plan: &SearchPlan,
        request: &SearchRequest,
        corpus: &PreparedCorpus,
    ) -> Result<CandidateList> {
        // 1. Expand query
        let expander = self
            .expanders
            .get(&plan.query_expansion)
            .ok_or_else(|| anyhow!("expander not registered for {:?}", plan.query_expansion))?;

        let query_variants = expander.expand(&request.query);

        // 2. Retrieve candidates
        let mut all_lists = Vec::new();
        if request.retriever_timeout_ms.is_none() || plan.retrievers.len() <= 1 {
            for policy in &plan.retrievers {
                let retriever = self
                    .retrievers
                    .get(policy)
                    .ok_or_else(|| anyhow!("retriever not registered for {:?}", policy))?;

                let retriever_start = Instant::now();
                let list = retriever.retrieve(
                    &query_variants,
                    corpus,
                    request.shortlist,
                    request.verbose,
                )?;
                tracing::debug!(
                    policy = ?policy,
                    elapsed_ms = retriever_start.elapsed().as_millis(),
                    results = list.results.len(),
                    "retriever completed"
                );
                all_lists.push(list);
            }
        } else {
            let timeout_ms = request.retriever_timeout_ms.unwrap_or(u64::MAX);
            let timeout = Duration::from_millis(timeout_ms);
            let (tx, rx) = mpsc::channel();
            let shared_artifacts = std::sync::Arc::new(corpus.artifacts.to_vec());
            let shared_index = corpus.bm25_index.cloned().map(std::sync::Arc::new);
            let mut pending = HashSet::new();

            tracing::info!(
                plan_retrievers = ?plan.retrievers,
                timeout_ms = request.retriever_timeout_ms,
                shortlist = request.shortlist,
                "running retrievers in parallel with timeout fallback"
            );

            for policy in &plan.retrievers {
                let retriever = self
                    .retrievers
                    .get(policy)
                    .ok_or_else(|| anyhow!("retriever not registered for {:?}", policy))?
                    .clone();
                pending.insert(*policy);

                let tx = tx.clone();
                let policy = *policy;
                let query_variants = query_variants.clone();
                let shared_artifacts = std::sync::Arc::clone(&shared_artifacts);
                let shared_index = shared_index.clone();
                let shortlist = request.shortlist;
                let verbose = request.verbose;

                thread::spawn(move || {
                    let prepared = PreparedCorpus {
                        artifacts: &shared_artifacts,
                        bm25_index: shared_index.as_deref(),
                    };
                    let retriever_start = Instant::now();
                    let result = retriever.retrieve(&query_variants, &prepared, shortlist, verbose);
                    if let Err(error) = tx.send((policy, retriever_start.elapsed(), result)) {
                        tracing::warn!(
                            policy = ?policy,
                            error = %error,
                            "failed to deliver retriever result"
                        );
                    }
                });
            }

            drop(tx);
            let started = Instant::now();
            while !pending.is_empty() {
                let remaining = timeout.checked_sub(started.elapsed()).unwrap_or_default();
                if remaining == Duration::ZERO {
                    tracing::warn!(
                        requested_ms = timeout_ms,
                        pending = ?pending,
                        completed = plan.retrievers.len().saturating_sub(pending.len()),
                        "retriever strategy timeout reached; returning partial results"
                    );
                    break;
                }

                match rx.recv_timeout(remaining) {
                    Ok((policy, elapsed, result)) => {
                        pending.remove(&policy);
                        tracing::info!(
                            policy = ?policy,
                            elapsed_ms = elapsed.as_millis(),
                            completed = plan.retrievers.len().saturating_sub(pending.len()),
                            total = plan.retrievers.len(),
                            "retriever completed within timeout"
                        );
                        all_lists.push(result?);
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        tracing::warn!(
                            requested_ms = timeout_ms,
                            pending = ?pending,
                            completed = plan.retrievers.len().saturating_sub(pending.len()),
                            "retriever strategy timeout reached; returning partial results"
                        );
                        break;
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        tracing::warn!(
                            "retriever channel disconnected before all policies completed"
                        );
                        break;
                    }
                }
            }

            while let Ok((_policy, _elapsed, result)) = rx.try_recv() {
                all_lists.push(result?);
            }
        }

        if request.retriever_timeout_ms.is_some()
            && all_lists.is_empty()
            && let Some(retriever) = self.retrievers.get(&RetrieverPolicy::Bm25)
        {
            tracing::warn!(
                "all timed-out retrievers returned no results; falling back to BM25 only"
            );
            let fallback_start = Instant::now();
            let list =
                retriever.retrieve(&query_variants, corpus, request.shortlist, request.verbose)?;
            tracing::info!(
                elapsed_ms = fallback_start.elapsed().as_millis(),
                results = list.results.len(),
                "bm25 fallback completed"
            );
            all_lists.push(list);
        }

        // 3. Fuse candidates
        let fuser = self
            .fusers
            .get(&plan.fusion)
            .ok_or_else(|| anyhow!("fuser not registered for {:?}", plan.fusion))?;

        let fused = fuser.fuse(&all_lists, request.shortlist, request.verbose)?;

        // 4. Rerank candidates
        let reranker = self
            .rerankers
            .get(&plan.reranking)
            .ok_or_else(|| anyhow!("reranker not registered for {:?}", plan.reranking))?;

        // Construct rerank query
        let rerank_query = if let Some(intent) = request.intent.as_deref() {
            format!("{}. Intent: {}", request.query, intent)
        } else {
            request.query.clone()
        };

        reranker.rerank(&rerank_query, fused, request.limit)
    }
}

pub struct SearchServiceBuilder;

impl SearchServiceBuilder {
    pub fn load_llm_reranker(
        plan: &SearchPlan,
        request: &SearchRequest,
    ) -> Result<Option<Arc<dyn Reranker>>> {
        let mut llm_reranker = if plan.reranking == RerankingPolicy::Llm {
            if let Some(spec) = &request.rerank_model {
                Some(Arc::new(QwenReranker::load(spec.clone())?) as Arc<dyn Reranker>)
            } else {
                Some(Arc::new(QwenReranker::load(QwenModelSpec::default())?) as Arc<dyn Reranker>)
            }
        } else if plan.reranking == RerankingPolicy::Jina {
            Some(Arc::new(JinaReranker::load(JinaModelSpec::default())?) as Arc<dyn Reranker>)
        } else if plan.reranking == RerankingPolicy::Gemma {
            if let Some(spec) = &request.gemma_model {
                Some(Arc::new(GemmaReranker::load(spec.clone())?) as Arc<dyn Reranker>)
            } else {
                Some(Arc::new(GemmaReranker::load(GemmaModelSpec::default())?) as Arc<dyn Reranker>)
            }
        } else {
            None
        };

        // If we need generative expansion but don't have a reranker (or it's not generative),
        // load the default Instruct model.
        let expansion_needs_llm = matches!(
            plan.query_expansion,
            QueryExpansionPolicy::Hyde
                | QueryExpansionPolicy::Splade
                | QueryExpansionPolicy::Classified
        );

        if llm_reranker.is_none() && expansion_needs_llm {
            tracing::info!("loading Instruct model for query expansion...");
            llm_reranker =
                Some(Arc::new(QwenReranker::load(QwenModelSpec::default())?) as Arc<dyn Reranker>);
        }

        Ok(llm_reranker)
    }

    pub fn build(
        plan: &SearchPlan,
        embedder: Option<Arc<dyn Embedder>>,
        query_cache: Option<QueryEmbeddingCache>,
        llm_reranker: Option<Arc<dyn Reranker>>,
        prompts: Option<&crate::config::PromptsConfig>,
    ) -> SearchService {
        let mut service = SearchService::new();

        service.register_fuser(FusionPolicy::Rrf, Box::new(RrfFuser));
        service.register_expander(QueryExpansionPolicy::None, Box::new(NoExpander));
        service.register_expander(QueryExpansionPolicy::Synonym, Box::new(SynonymExpander));

        let mut hyde = LlmExpander::new(Box::new(HydeStrategy {
            custom_prompt: prompts.and_then(|p| p.hyde.clone()),
        }));
        let mut splade = LlmExpander::new(Box::new(SpladeStrategy {
            custom_prompt: prompts.and_then(|p| p.splade.clone()),
        }));
        let mut classified = LlmExpander::new(Box::new(ClassifiedStrategy {
            custom_prompt: prompts.and_then(|p| p.classified.clone()),
        }));

        if let Some(r) = &llm_reranker {
            let generative = Arc::new(RerankerAsGenerative(r.clone())) as Arc<dyn GenerativeModel>;
            hyde = hyde.with_llm(generative.clone());
            splade = splade.with_llm(generative.clone());
            classified = classified.with_llm(generative);
        }

        service.register_expander(QueryExpansionPolicy::Hyde, Box::new(hyde));
        service.register_expander(QueryExpansionPolicy::Splade, Box::new(splade));
        service.register_expander(QueryExpansionPolicy::Classified, Box::new(classified));

        service.register_reranker(RerankingPolicy::None, Box::new(NoReranker));
        service.register_reranker(
            RerankingPolicy::PositionAware,
            Box::new(PositionAwareReranker),
        );

        if let Some(r) = llm_reranker {
            if plan.reranking == RerankingPolicy::Jina {
                service.register_reranker_arc(RerankingPolicy::Jina, r);
            } else if plan.reranking == RerankingPolicy::Gemma {
                service.register_reranker_arc(RerankingPolicy::Gemma, r);
            } else {
                service.register_reranker_arc(RerankingPolicy::Llm, r);
            }
        } else {
            service.register_reranker(RerankingPolicy::Llm, Box::new(MockLlmReranker));
            service.register_reranker(RerankingPolicy::Jina, Box::new(MockLlmReranker));
            service.register_reranker(RerankingPolicy::Gemma, Box::new(MockLlmReranker));
        }

        // Register retrievers
        service.register_retriever(Box::new(Bm25Retriever));
        service.register_retriever(Box::new(PhraseRetriever));
        service.register_retriever(Box::new(PathFuzzyRetriever));
        service.register_retriever(Box::new(SegmentFuzzyRetriever));
        if let Some(e) = embedder {
            let final_embedder = if let Some(cache) = query_cache {
                Arc::new(crate::search::domain::CachedEmbedder { inner: e, cache })
                    as Arc<dyn Embedder>
            } else {
                e
            };
            service.register_retriever(Box::new(SegmentVectorRetriever {
                embedder: final_embedder,
            }));
        }

        service
    }
}

pub fn run_search(
    request: &SearchRequest,
    ignore: Option<&Ignore>,
    repository: &dyn CorpusRepository,
    embedder: Option<Arc<dyn Embedder>>,
) -> Result<SearchResponse> {
    let registry = StrategyPresetRegistry::default_registry();
    let mut plan = registry.resolve(&request.strategy)?;

    // Apply overrides from SearchRequest
    if let Some(retrievers) = &request.retrievers {
        plan.retrievers = retrievers.clone();
    }
    if let Some(fusion) = request.fusion {
        plan.fusion = fusion;
    }
    if let Some(reranking) = request.reranking {
        plan.reranking = reranking;
    }

    run_search_with_plan(&plan, request, ignore, repository, embedder)
}

pub fn run_search_with_plan(
    plan: &SearchPlan,
    request: &SearchRequest,
    ignore: Option<&Ignore>,
    repository: &dyn CorpusRepository,
    embedder: Option<Arc<dyn Embedder>>,
) -> Result<SearchResponse> {
    run_search_with_plan_and_progress(plan, request, ignore, repository, embedder, None)
}

pub(crate) struct PreparedSearchRuntime {
    pub corpus: LoadedCorpus,
    pub index: Bm25Index,
    pub total_chunks: usize,
}

pub(crate) fn prepare_search_runtime_with_progress(
    request: &SearchRequest,
    ignore: Option<&Ignore>,
    repository: &dyn CorpusRepository,
    embedder: Option<Arc<dyn Embedder>>,
    progress: Option<&dyn Fn(&SearchProgress)>,
) -> Result<PreparedSearchRuntime> {
    let verbose = request.verbose;

    let corpus_start = std::time::Instant::now();
    let corpus = repository.load_with_progress(
        &crate::search::CorpusLoadRequest {
            path: &request.path,
            ignore,
            verbose,
            embedder: embedder.as_deref(),
            telemetry: &request.telemetry,
            local_context: &request.local_context,
            cache_dir: request.cache_dir.as_deref(),
        },
        progress,
    )?;
    tracing::info!(
        "corpus loaded ({} artifacts) in {:.2?}",
        corpus.indexed_artifacts,
        corpus_start.elapsed()
    );

    let total_chunks: usize = corpus
        .artifacts
        .iter()
        .map(|artifact| artifact.segments.len())
        .sum();
    let index = prepare_bm25_index(
        &request.path,
        &corpus,
        request.cache_dir.as_deref(),
        &request.telemetry,
        progress,
    )?;

    Ok(PreparedSearchRuntime {
        corpus,
        index,
        total_chunks,
    })
}

pub fn run_search_with_plan_and_progress(
    plan: &SearchPlan,
    request: &SearchRequest,
    ignore: Option<&Ignore>,
    repository: &dyn CorpusRepository,
    embedder: Option<Arc<dyn Embedder>>,
    progress: Option<&dyn Fn(&SearchProgress)>,
) -> Result<SearchResponse> {
    let prepared = prepare_search_runtime_with_progress(
        request,
        ignore,
        repository,
        embedder.clone(),
        progress,
    )?;
    let PreparedSearchRuntime {
        corpus,
        index,
        total_chunks,
    } = prepared;

    let llm_reranker = SearchServiceBuilder::load_llm_reranker(plan, request)?;

    let service = SearchServiceBuilder::build(
        plan,
        embedder,
        request.query_cache.clone(),
        llm_reranker,
        request.prompts.as_ref(),
    );

    let prepared = PreparedCorpus {
        artifacts: &corpus.artifacts,
        bm25_index: Some(&index),
    };

    if plan_uses_vector_retriever(plan)
        && let Some(cb) = progress
    {
        cb(&SearchProgress::Embedding {
            phase: SearchPhase::Embedding,
            chunks_processed: 0,
            chunks_total: total_chunks,
            estimated_remaining: None,
        });
    }

    let candidates = service.execute(plan, request, &prepared)?;

    if plan_uses_vector_retriever(plan)
        && let Some(cb) = progress
    {
        cb(&SearchProgress::Embedding {
            phase: SearchPhase::Embedding,
            chunks_processed: total_chunks,
            chunks_total: total_chunks,
            estimated_remaining: Some(Duration::from_secs(0)),
        });
    }

    let hits = project_hits(plan, &corpus, candidates.results, &request.query);

    if let Some(cb) = progress {
        cb(&SearchProgress::Ranking {
            phase: SearchPhase::Ranking,
            results_processed: hits.len(),
            results_total: hits.len(),
            estimated_remaining: Some(Duration::from_secs(0)),
        });
    }

    Ok(SearchResponse {
        strategy: plan.name.clone(),
        root: request.path.display().to_string(),
        indexed_artifacts: corpus.indexed_artifacts,
        skipped_artifacts: corpus.skipped_artifacts,
        coverage: SearchCoverageSnapshot::from_frontier(&request.telemetry.frontier_snapshot()),
        hits,
    })
}

pub(crate) fn prepare_bm25_index(
    root: &std::path::Path,
    corpus: &LoadedCorpus,
    cache_dir: Option<&std::path::Path>,
    telemetry: &crate::system::Telemetry,
    progress: Option<&dyn Fn(&SearchProgress)>,
) -> Result<Bm25Index> {
    let index_start = std::time::Instant::now();
    let total_files = telemetry.total_files.load(Ordering::Relaxed);
    let emit_indexing_complete = |progress: Option<&dyn Fn(&SearchProgress)>| {
        if let Some(cb) = progress {
            cb(&SearchProgress::Indexing {
                phase: SearchPhase::Indexing,
                files_processed: total_files,
                files_total: total_files,
                estimated_remaining: Some(Duration::from_secs(0)),
                coverage: SearchCoverageSnapshot::from_frontier(&telemetry.frontier_snapshot()),
            });
        }
    };

    let index = if let Some(cache_dir) = cache_dir {
        let cache_root = resolve_compatible_cache_path(root);
        let signature = crate::search::corpus::compute_bm25_index_signature(&corpus.artifacts);

        match crate::search::corpus::load_bm25_index_cache(cache_dir, &cache_root, &signature) {
            Ok(Some(cached)) => {
                telemetry
                    .bm25_index_cache_hits
                    .fetch_add(1, Ordering::Relaxed);
                tracing::info!(
                    "loaded cached bm25 index in {:.2?} (signature={})",
                    index_start.elapsed(),
                    signature
                );
                cached
            }
            Ok(None) => {
                tracing::info!(
                    "bm25 index cache miss for signature {}; building index",
                    signature
                );
                let built =
                    load_or_build_sector_bm25_index(cache_dir, &cache_root, corpus, telemetry)?;
                if let Err(error) = crate::search::corpus::save_bm25_index_cache(
                    cache_dir,
                    &cache_root,
                    &signature,
                    &built,
                ) {
                    tracing::warn!(
                        error = %error,
                        signature = %signature,
                        "failed to save bm25 index cache"
                    );
                }
                built
            }
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    "bm25 cache read failed; rebuilding index"
                );
                let built =
                    load_or_build_sector_bm25_index(cache_dir, &cache_root, corpus, telemetry)?;
                if let Err(error) = crate::search::corpus::save_bm25_index_cache(
                    cache_dir,
                    &cache_root,
                    &signature,
                    &built,
                ) {
                    tracing::warn!(
                        error = %error,
                        signature = %signature,
                        "failed to save rebuilt bm25 index"
                    );
                }
                built
            }
        }
    } else {
        telemetry.bm25_index_builds.fetch_add(1, Ordering::Relaxed);
        Bm25Index::build(&corpus.artifacts)
    };

    tracing::info!("bm25 index ready in {:.2?}", index_start.elapsed());
    emit_indexing_complete(progress);

    Ok(index)
}

fn load_or_build_sector_bm25_index(
    cache_dir: &std::path::Path,
    root: &std::path::Path,
    corpus: &LoadedCorpus,
    telemetry: &crate::system::Telemetry,
) -> Result<Bm25Index> {
    let sector_map = crate::cache::SectorMap::load_for_root(cache_dir, root).unwrap_or_default();
    let mut shards = Vec::new();

    for sector in &sector_map.sectors {
        let Some(shard_ref) = sector.shards.bm25.as_ref() else {
            telemetry.bm25_index_builds.fetch_add(1, Ordering::Relaxed);
            return Ok(Bm25Index::build(&corpus.artifacts));
        };

        let Some(shard) = crate::search::corpus::load_sector_bm25_shard(
            cache_dir,
            root,
            &sector.sector_id,
            &shard_ref.key,
        )?
        else {
            telemetry.bm25_index_builds.fetch_add(1, Ordering::Relaxed);
            return Ok(Bm25Index::build(&corpus.artifacts));
        };

        telemetry
            .sector_shard_cache_hits
            .fetch_add(1, Ordering::Relaxed);
        shards.push(shard);
    }

    if shards.is_empty() {
        telemetry.bm25_index_builds.fetch_add(1, Ordering::Relaxed);
        Ok(Bm25Index::build(&corpus.artifacts))
    } else {
        Ok(combine_bm25_shards(&shards))
    }
}

fn combine_bm25_shards(shards: &[Bm25Index]) -> Bm25Index {
    let mut doc_freq = std::collections::HashMap::new();
    let mut term_freqs = std::collections::HashMap::new();
    let mut doc_lengths = std::collections::HashMap::new();
    let mut total_length = 0usize;
    let mut num_docs = 0usize;

    for shard in shards {
        for (term, frequency) in &shard.doc_freq {
            *doc_freq.entry(term.clone()).or_insert(0) += frequency;
        }
        for (doc_id, frequencies) in &shard.term_freqs {
            term_freqs.insert(doc_id.clone(), frequencies.clone());
        }
        for (doc_id, length) in &shard.doc_lengths {
            doc_lengths.insert(doc_id.clone(), *length);
            total_length += *length;
            num_docs += 1;
        }
    }

    Bm25Index {
        doc_freq,
        term_freqs,
        doc_lengths,
        avg_doc_len: if num_docs == 0 {
            0.0
        } else {
            total_length as f64 / num_docs as f64
        },
        num_docs,
    }
}

fn plan_uses_vector_retriever(plan: &SearchPlan) -> bool {
    plan.retrievers
        .iter()
        .any(|policy| matches!(policy, RetrieverPolicy::Vector))
}

pub fn project_hits(
    plan: &SearchPlan,
    corpus: &LoadedCorpus,
    results: Vec<Candidate>,
    query: &str,
) -> Vec<SearchHit> {
    let mut hits = Vec::with_capacity(results.len());

    for result in results {
        let Some(artifact) = corpus.artifact_by_id(&result.id) else {
            tracing::warn!(
                candidate_id = %result.id,
                candidate_path = %result.path.display(),
                "skipping candidate missing from loaded corpus"
            );
            continue;
        };

        let mut path = result.path.display().to_string();
        if path.starts_with("./") {
            path = path.chars().skip(2).collect();
        }

        hits.push(SearchHit {
            artifact_id: artifact.id.clone(),
            artifact_kind: artifact.kind,
            path,
            rank: hits.len() + 1,
            score: result.score,
            confidence: plan.categorize_score(result.score),
            location: result.snippet_location.clone(),
            snippet: resolve_snippet_from_candidate(corpus, &result, query),
            provenance: artifact.provenance.clone(),
            freshness: artifact.freshness.clone(),
            budget: artifact.budget.clone(),
        });
    }

    hits
}

pub fn resolve_snippet_from_candidate(
    corpus: &LoadedCorpus,
    candidate: &Candidate,
    query: &str,
) -> String {
    if let Some(snippet) = &candidate.snippet {
        return super::presentation::build_snippet(snippet, query);
    }

    let artifact = match corpus.artifact_by_id(&candidate.id) {
        Some(artifact) => artifact,
        None => return String::new(),
    };

    let terms = tokenize(query);
    let mut best_snippet = String::new();
    let mut max_matches = 0;

    for segment in &artifact.segments {
        let text = &segment.text;
        let matches = terms
            .iter()
            .filter(|term| text.to_lowercase().contains(*term))
            .count();

        if matches > max_matches {
            max_matches = matches;
            best_snippet = text.to_string();
        }
    }

    if best_snippet.is_empty() && !artifact.segments.is_empty() {
        best_snippet = artifact.segments[0].text.to_string();
    }

    if best_snippet.is_empty() {
        super::presentation::build_snippet(artifact.text(), query)
    } else {
        super::presentation::build_snippet(&best_snippet, query)
    }
}

pub struct LocalFileCorpusRepository;

impl CorpusRepository for LocalFileCorpusRepository {
    fn load(&self, request: &crate::search::CorpusLoadRequest<'_>) -> Result<LoadedCorpus> {
        crate::internal::search::corpus::load_search_corpus(
            request.path,
            request.ignore,
            request.verbose,
            request.embedder,
            request.telemetry,
            request.local_context,
            request.cache_dir,
        )
    }

    fn load_with_progress(
        &self,
        request: &crate::search::CorpusLoadRequest<'_>,
        progress: Option<&dyn Fn(&SearchProgress)>,
    ) -> Result<LoadedCorpus> {
        crate::internal::search::corpus::load_search_corpus_with_progress(
            request.path,
            request.ignore,
            request.verbose,
            request.embedder,
            request.telemetry,
            request.local_context,
            request.cache_dir,
            progress,
        )
    }
}
