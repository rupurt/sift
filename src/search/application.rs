use super::adapters::*;
use super::domain::{
    Bm25Index, Candidate, CandidateList, CorpusRepository, Embedder, Expander, Fuser, FusionPolicy,
    GenerativeModel, LoadedCorpus, PreparedCorpus, QueryEmbeddingCache, QueryExpansionPolicy,
    Reranker, RerankingPolicy, Retriever, RetrieverPolicy, SearchHit, SearchPlan, SearchRequest,
    SearchResponse, StrategyPresetRegistry, tokenize,
};
use crate::config::Ignore;
use anyhow::{Result, anyhow};
use std::sync::Arc;

pub struct SearchService {
    retrievers: std::collections::HashMap<RetrieverPolicy, Box<dyn Retriever>>,
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
        self.retrievers.insert(retriever.policy(), retriever);
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
        for policy in &plan.retrievers {
            let retriever = self
                .retrievers
                .get(policy)
                .ok_or_else(|| anyhow!("retriever not registered for {:?}", policy))?;

            all_lists.push(retriever.retrieve(
                &query_variants,
                corpus,
                request.shortlist,
                request.verbose,
            )?);
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
    let verbose = request.verbose;

    // For SegmentVectorRetriever, we need to load the model if the plan requires it
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

    let corpus_start = std::time::Instant::now();
    let corpus = repository.load(
        &request.path,
        ignore,
        verbose,
        embedder.as_deref(),
        &request.telemetry,
        request.cache_dir.as_deref(),
    )?;
    tracing::info!(
        "corpus loaded ({} files) in {:.2?}",
        corpus.indexed_files,
        corpus_start.elapsed()
    );

    let llm_reranker = SearchServiceBuilder::load_llm_reranker(&plan, request)?;

    let service = SearchServiceBuilder::build(
        &plan,
        embedder,
        request.query_cache.clone(),
        llm_reranker,
        request.prompts.as_ref(),
    );

    let index_start = std::time::Instant::now();
    let index = Bm25Index::build(&corpus.documents);
    tracing::info!("built bm25 index in {:.2?}", index_start.elapsed());
    let prepared = PreparedCorpus {
        documents: &corpus.documents,
        bm25_index: Some(&index),
    };

    let candidates = service.execute(&plan, request, &prepared)?;

    let results = candidates
        .results
        .into_iter()
        .enumerate()
        .map(|(index, result)| {
            let mut path = result.path.display().to_string();
            if path.starts_with("./") {
                path = path.chars().skip(2).collect();
            }
            SearchHit {
                path,
                rank: index + 1,
                score: result.score,
                confidence: plan.categorize_score(result.score),
                location: result.snippet_location.clone(),
                snippet: resolve_snippet_from_candidate(&corpus, &result, &request.query),
            }
        })
        .collect();

    Ok(SearchResponse {
        strategy: request.strategy.clone(),
        root: request.path.display().to_string(),
        indexed_files: corpus.indexed_files,
        skipped_files: corpus.skipped_files,
        results,
    })
}

pub fn resolve_snippet_from_candidate(
    corpus: &LoadedCorpus,
    candidate: &Candidate,
    query: &str,
) -> String {
    if let Some(snippet) = &candidate.snippet {
        return super::presentation::build_snippet(snippet, query);
    }

    let document = match corpus.document_by_id(&candidate.id) {
        Some(doc) => doc,
        None => return String::new(),
    };

    let terms = tokenize(query);
    let mut best_snippet = String::new();
    let mut max_matches = 0;

    for segment in &document.segments {
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

    if best_snippet.is_empty() && !document.segments.is_empty() {
        best_snippet = document.segments[0].text.to_string();
    }

    if best_snippet.is_empty() {
        super::presentation::build_snippet(document.text(), query)
    } else {
        super::presentation::build_snippet(&best_snippet, query)
    }
}

pub struct LocalFileCorpusRepository;

impl CorpusRepository for LocalFileCorpusRepository {
    fn load(
        &self,
        path: &std::path::Path,
        ignore: Option<&Ignore>,
        verbose: u8,
        embedder: Option<&dyn Embedder>,
        telemetry: &crate::system::Telemetry,
        cache_dir: Option<&std::path::Path>,
    ) -> Result<LoadedCorpus> {
        crate::internal::search::corpus::load_search_corpus(
            path, ignore, verbose, embedder, telemetry, cache_dir,
        )
    }
}
