use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;

use super::adapters::jina::{JinaModelSpec, JinaReranker};
use super::adapters::qwen::{QwenModelSpec, QwenReranker};
use super::adapters::*;
use super::domain::*;
use crate::config::Ignore;

pub struct SearchServiceBuilder;

impl SearchServiceBuilder {
    pub fn build(
        plan: &SearchPlan,
        embedder: Option<Arc<dyn Embedder>>,
        query_cache: Option<QueryEmbeddingCache>,
        llm_reranker: Option<Arc<dyn Reranker>>,
    ) -> SearchService {
        let mut service = SearchService::new();

        service.register_fuser(FusionPolicy::Rrf, Box::new(RrfFuser));
        service.register_expander(QueryExpansionPolicy::None, Box::new(NoExpander));
        service.register_expander(QueryExpansionPolicy::Synonym, Box::new(SynonymExpander));

        let mut hyde = LlmExpander::new(Box::new(HydeStrategy));
        let mut splade = LlmExpander::new(Box::new(SpladeStrategy));
        let mut classified = LlmExpander::new(Box::new(ClassifiedStrategy));

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
            } else {
                service.register_reranker_arc(RerankingPolicy::Llm, r);
            }
        } else {
            service.register_reranker(RerankingPolicy::Llm, Box::new(MockLlmReranker));
            service.register_reranker(RerankingPolicy::Jina, Box::new(MockLlmReranker));
        }

        // Register retrievers based on plan
        if plan.retrievers.contains(&RetrieverPolicy::Bm25) {
            service.register_retriever(Box::new(Bm25Retriever));
        }
        if plan.retrievers.contains(&RetrieverPolicy::Phrase) {
            service.register_retriever(Box::new(PhraseRetriever));
        }
        if plan.retrievers.contains(&RetrieverPolicy::Vector)
            && let Some(e) = embedder
        {
            let final_embedder = if let Some(cache) = query_cache {
                Arc::new(CachedEmbedder { inner: e, cache }) as Arc<dyn Embedder>
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

pub struct SearchService {
    retrievers: HashMap<RetrieverPolicy, Box<dyn Retriever>>,
    fusers: HashMap<FusionPolicy, Box<dyn Fuser>>,
    expanders: HashMap<QueryExpansionPolicy, Box<dyn Expander>>,
    rerankers: HashMap<RerankingPolicy, Arc<dyn Reranker>>,
}

pub struct StrategyPresetRegistry {
    presets: HashMap<String, SearchPlan>,
    champion: String,
}

impl StrategyPresetRegistry {
    pub fn new(champion: &str) -> Self {
        Self {
            presets: HashMap::new(),
            champion: champion.to_string(),
        }
    }

    pub fn default_registry() -> Self {
        let mut registry = Self::new("page-index-hybrid");

        // bm25 preset
        registry.register(
            "bm25",
            SearchPlan {
                name: "bm25".to_string(),
                query_expansion: QueryExpansionPolicy::None,
                retrievers: vec![RetrieverPolicy::Bm25],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::None,
            },
        );

        // vector preset
        registry.register(
            "vector",
            SearchPlan {
                name: "vector".to_string(),
                query_expansion: QueryExpansionPolicy::None,
                retrievers: vec![RetrieverPolicy::Vector],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::None,
            },
        );

        // legacy-hybrid preset
        registry.register(
            "legacy-hybrid",
            SearchPlan {
                name: "legacy-hybrid".to_string(),
                query_expansion: QueryExpansionPolicy::None,
                retrievers: vec![RetrieverPolicy::Bm25, RetrieverPolicy::Vector],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::None,
            },
        );

        // page-index (lexical focus, inspired by qmd)
        registry.register(
            "page-index",
            SearchPlan {
                name: "page-index".to_string(),
                query_expansion: QueryExpansionPolicy::None,
                retrievers: vec![RetrieverPolicy::Bm25, RetrieverPolicy::Phrase],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::PositionAware,
            },
        );

        // page-index-hybrid (champion, lexical + vector)
        registry.register(
            "page-index-hybrid",
            SearchPlan {
                name: "page-index-hybrid".to_string(),
                query_expansion: QueryExpansionPolicy::Splade,
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                    RetrieverPolicy::Vector,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::PositionAware,
            },
        );

        // page-index-llm (lexical + vector + llm reranking)
        registry.register(
            "page-index-llm",
            SearchPlan {
                name: "page-index-llm".to_string(),
                query_expansion: QueryExpansionPolicy::Hyde,
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                    RetrieverPolicy::Vector,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::Llm,
            },
        );

        // page-index-qwen (alias for page-index-llm for explicit Qwen testing)
        registry.register(
            "page-index-qwen",
            SearchPlan {
                name: "page-index-qwen".to_string(),
                query_expansion: QueryExpansionPolicy::None,
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                    RetrieverPolicy::Vector,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::Llm,
            },
        );

        // page-index-splade (generative expansion focus)
        registry.register(
            "page-index-splade",
            SearchPlan {
                name: "page-index-splade".to_string(),
                query_expansion: QueryExpansionPolicy::Splade,
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                    RetrieverPolicy::Vector,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::PositionAware,
            },
        );

        // page-index-classified (classification-driven expansion)
        registry.register(
            "page-index-classified",
            SearchPlan {
                name: "page-index-classified".to_string(),
                query_expansion: QueryExpansionPolicy::Classified,
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                    RetrieverPolicy::Vector,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::PositionAware,
            },
        );

        // page-index-jina (Jina Reranker v3)
        registry.register(
            "page-index-jina",
            SearchPlan {
                name: "page-index-jina".to_string(),
                query_expansion: QueryExpansionPolicy::Splade,
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                    RetrieverPolicy::Vector,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::Jina,
            },
        );

        registry
    }

    pub fn register(&mut self, name: &str, plan: SearchPlan) {
        self.presets.insert(name.to_string(), plan);
    }

    pub fn resolve(&self, name: &str) -> Result<SearchPlan> {
        let actual_name = if name == "hybrid" {
            &self.champion
        } else {
            name
        };

        self.presets
            .get(actual_name)
            .cloned()
            .ok_or_else(|| anyhow!("unknown search strategy: {}", name))
    }

    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.presets.keys().cloned().collect();
        names.sort();
        names
    }
}

impl Default for SearchService {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchService {
    pub fn new() -> Self {
        Self {
            retrievers: HashMap::new(),
            fusers: HashMap::new(),
            expanders: HashMap::new(),
            rerankers: HashMap::new(),
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

    #[allow(clippy::too_many_arguments)]
    pub fn execute(
        &self,
        plan: &SearchPlan,
        query: &str,
        intent: Option<&str>,
        corpus: &PreparedCorpus,
        limit: usize,
        shortlist: usize,
        _verbose: u8,
        telemetry: &crate::system::Telemetry,
    ) -> Result<CandidateList> {
        let start = std::time::Instant::now();
        tracing::info!("executing strategy: {} for query: '{}'", plan.name, query);

        if let Some(intent_text) = intent {
            tracing::info!("using provided intent: '{}'", intent_text);
        }

        let retrieval_limit = std::cmp::max(limit, shortlist);

        // 1. Query Expansion
        let expand_start = std::time::Instant::now();

        let effective_policy = if intent.is_some() {
            QueryExpansionPolicy::None
        } else if plan.reranking == RerankingPolicy::Llm {
            QueryExpansionPolicy::Hyde
        } else {
            QueryExpansionPolicy::Splade
        };

        let expander = self
            .expanders
            .get(&effective_policy)
            .or_else(|| self.expanders.get(&plan.query_expansion))
            .ok_or_else(|| anyhow!("expander not found for policy: {:?}", effective_policy))?;

        let query_variants = if let Some(intent_text) = intent {
            vec![QueryVariant {
                text: format!("{} {}", query, intent_text),
                weight: 1.0,
            }]
        } else {
            expander.expand(query)
        };
        tracing::info!(
            "expanded query into {} variants in {:.2?}",
            query_variants.len(),
            expand_start.elapsed()
        );
        tracing::debug!(
            "variants: {:?}",
            query_variants.iter().map(|v| &v.text).collect::<Vec<_>>()
        );

        // 2. Retrieval
        let retrieval_start = std::time::Instant::now();
        let mut candidate_lists = Vec::new();
        for policy in &plan.retrievers {
            let retrieve_start = std::time::Instant::now();
            let retriever = self
                .retrievers
                .get(policy)
                .ok_or_else(|| anyhow!("retriever not found for policy: {:?}", policy))?;
            // Note: adapters will be updated to use tracing internally later, for now we pass 0 as verbose
            let list = retriever.retrieve(&query_variants, corpus, retrieval_limit, 0)?;
            tracing::info!(
                "{:?}: found {} candidates in {:.2?}",
                policy,
                list.results.len(),
                retrieve_start.elapsed()
            );
            candidate_lists.push(list);
        }
        tracing::info!("retrieval complete in {:.2?}", retrieval_start.elapsed());

        if candidate_lists.is_empty() {
            return Ok(CandidateList {
                results: Vec::new(),
            });
        }

        // 3. Fusion
        let fuse_start = std::time::Instant::now();
        let fuser = self
            .fusers
            .get(&plan.fusion)
            .ok_or_else(|| anyhow!("fuser not found for policy: {:?}", plan.fusion))?;
        let fused = fuser.fuse(&candidate_lists, retrieval_limit, 0)?;
        tracing::info!(
            "fused into {} candidates in {:.2?}",
            fused.results.len(),
            fuse_start.elapsed()
        );

        // 4. Reranking
        let rerank_start = std::time::Instant::now();
        let reranker = self
            .rerankers
            .get(&plan.reranking)
            .ok_or_else(|| anyhow!("reranker not found for policy: {:?}", plan.reranking))?;
        let mut final_list = if shortlist < fused.results.len() {
            let mut shortlist_results = fused.results;
            let remaining = shortlist_results.split_off(shortlist);

            let mut final_list = reranker.rerank(
                query,
                CandidateList {
                    results: shortlist_results,
                },
                limit.min(shortlist),
            )?;

            let missing = limit.saturating_sub(final_list.results.len());
            final_list
                .results
                .extend(remaining.into_iter().take(missing));

            final_list
        } else {
            reranker.rerank(query, fused, limit.min(shortlist))?
        };

        if final_list.results.len() > limit {
            final_list.results.truncate(limit);
        }
        tracing::info!("reranking complete in {:.2?}", rerank_start.elapsed());

        tracing::info!("search complete in {:.2?}", start.elapsed());
        telemetry.trace_hit_rates();
        Ok(final_list)
    }
}

pub struct SearchEnvironment<'a> {
    pub service: SearchService,
    pub plan: SearchPlan,
    pub corpus: &'a LoadedCorpus,
    pub prepared: PreparedCorpus<'a>,
    pub telemetry: std::sync::Arc<crate::system::Telemetry>,
}

impl<'a> SearchEnvironment<'a> {
    pub fn new(
        request: &SearchRequest,
        corpus: &'a LoadedCorpus,
        index: &'a Bm25Index,
        embedder: Option<Arc<dyn Embedder>>,
    ) -> Result<Self> {
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

        let service =
            SearchServiceBuilder::build(&plan, embedder, request.query_cache.clone(), None);

        Ok(Self {
            service,
            plan,
            corpus,
            prepared: PreparedCorpus {
                documents: &corpus.documents,
                bm25_index: Some(index),
            },
            telemetry: request.telemetry.clone(),
        })
    }

    pub fn search(
        &self,
        query: &str,
        intent: Option<&str>,
        limit: usize,
        shortlist: usize,
        verbose: u8,
    ) -> Result<SearchResponse> {
        let candidates = self.service.execute(
            &self.plan,
            query,
            intent,
            &self.prepared,
            limit,
            shortlist,
            verbose,
            &self.telemetry,
        )?;

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
                    confidence: self.plan.categorize_score(result.score),
                    location: result.snippet_location.clone(),
                    snippet: resolve_snippet_from_candidate(self.corpus, &result, query),
                }
            })
            .collect();

        Ok(SearchResponse {
            strategy: self.plan.name.clone(),
            root: String::new(), // Populated by caller
            indexed_files: self.corpus.indexed_files,
            skipped_files: self.corpus.skipped_files,
            results,
        })
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

    let mut llm_reranker = if plan.reranking == RerankingPolicy::Llm {
        if let Some(spec) = &request.rerank_model {
            Some(Arc::new(QwenReranker::load(spec.clone())?) as Arc<dyn Reranker>)
        } else {
            Some(Arc::new(QwenReranker::load(QwenModelSpec::default())?) as Arc<dyn Reranker>)
        }
    } else if plan.reranking == RerankingPolicy::Jina {
        Some(Arc::new(JinaReranker::load(JinaModelSpec::default())?) as Arc<dyn Reranker>)
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

    let service =
        SearchServiceBuilder::build(&plan, embedder, request.query_cache.clone(), llm_reranker);

    let index_start = std::time::Instant::now();
    let index = Bm25Index::build(&corpus.documents);
    tracing::info!("built bm25 index in {:.2?}", index_start.elapsed());
    let prepared = PreparedCorpus {
        documents: &corpus.documents,
        bm25_index: Some(&index),
    };

    let candidates = service.execute(
        &plan,
        &request.query,
        request.intent.as_deref(),
        &prepared,
        request.limit,
        request.shortlist,
        verbose,
        &request.telemetry,
    )?;

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

fn resolve_snippet_from_candidate(
    corpus: &LoadedCorpus,
    candidate: &Candidate,
    query: &str,
) -> String {
    if let Some(snippet) = candidate.snippet.as_deref() {
        return super::presentation::build_snippet(snippet, query);
    }

    super::presentation::build_snippet(
        corpus
            .document_by_id(&candidate.id)
            .map(Document::text)
            .unwrap_or_default(),
        query,
    )
}

#[cfg(test)]
mod tests {
    use super::super::adapters::*;
    use super::*;
    use std::sync::{Arc, Mutex};

    struct MockRetriever {
        policy: RetrieverPolicy,
    }

    impl Retriever for MockRetriever {
        fn retrieve(
            &self,
            variants: &[QueryVariant],
            _corpus: &PreparedCorpus,
            _limit: usize,
            _verbose: u8,
        ) -> Result<CandidateList> {
            let mut results = Vec::new();
            for variant in variants {
                results.push(Candidate {
                    id: variant.text.clone(),
                    path: std::path::PathBuf::from("mock"),
                    score: variant.weight,
                    contributors: vec![],
                    snippet: None,
                    snippet_location: None,
                });
            }
            Ok(CandidateList { results })
        }
        fn policy(&self) -> RetrieverPolicy {
            self.policy
        }
    }

    struct MockRankedRetriever {
        policy: RetrieverPolicy,
        count: usize,
    }

    impl Retriever for MockRankedRetriever {
        fn retrieve(
            &self,
            _variants: &[QueryVariant],
            _corpus: &PreparedCorpus,
            limit: usize,
            _verbose: u8,
        ) -> Result<CandidateList> {
            let mut results = Vec::new();
            for index in 0..self.count.min(limit) {
                results.push(Candidate {
                    id: format!("doc{index:02}"),
                    path: std::path::PathBuf::from("mock"),
                    score: (self.count - index) as f64,
                    contributors: vec![],
                    snippet: None,
                    snippet_location: None,
                });
            }
            Ok(CandidateList { results })
        }

        fn policy(&self) -> RetrieverPolicy {
            self.policy
        }
    }

    struct CapturingReranker {
        observed_input_sizes: Arc<Mutex<Vec<usize>>>,
    }

    impl CapturingReranker {
        fn new() -> (Self, Arc<Mutex<Vec<usize>>>) {
            let observed_input_sizes = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    observed_input_sizes: observed_input_sizes.clone(),
                },
                observed_input_sizes,
            )
        }
    }

    impl Reranker for CapturingReranker {
        fn rerank(
            &self,
            _query: &str,
            mut candidates: CandidateList,
            limit: usize,
        ) -> Result<CandidateList> {
            if let Ok(mut observed) = self.observed_input_sizes.lock() {
                observed.push(candidates.results.len());
            }

            candidates.results.sort_by(|a, b| b.id.cmp(&a.id));
            Ok(CandidateList {
                results: candidates.results.into_iter().take(limit).collect(),
            })
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn search_service_orchestrates_multiple_variants_and_retrievers() {
        let mut service = SearchService::new();
        service.register_expander(QueryExpansionPolicy::Synonym, Box::new(SynonymExpander));
        service.register_retriever(Box::new(MockRetriever {
            policy: RetrieverPolicy::Bm25,
        }));
        service.register_fuser(FusionPolicy::Rrf, Box::new(RrfFuser));
        service.register_reranker(RerankingPolicy::None, Box::new(NoReranker));

        let plan = SearchPlan {
            name: "test".to_string(),
            query_expansion: QueryExpansionPolicy::Synonym,
            retrievers: vec![RetrieverPolicy::Bm25],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::None,
        };

        let corpus = PreparedCorpus {
            documents: &[],
            bm25_index: None,
        };
        let telemetry = crate::system::Telemetry::new();
        let results = service
            .execute(&plan, "search", None, &corpus, 10, 10, 0, &telemetry)
            .unwrap();

        // "search" expansion with SynonymExpander gives "search" and "retrieval"
        // MockRetriever returns them as candidates
        assert!(results.results.iter().any(|c| c.id == "search"));
        assert!(results.results.iter().any(|c| c.id == "retrieval"));
    }

    #[test]
    fn search_service_uses_explicit_intent_for_expansion() {
        let mut service = SearchService::new();
        service.register_expander(QueryExpansionPolicy::None, Box::new(NoExpander));
        service.register_retriever(Box::new(MockRetriever {
            policy: RetrieverPolicy::Bm25,
        }));
        service.register_fuser(FusionPolicy::Rrf, Box::new(RrfFuser));
        service.register_reranker(RerankingPolicy::None, Box::new(NoReranker));

        let plan = SearchPlan {
            name: "test".to_string(),
            query_expansion: QueryExpansionPolicy::None,
            retrievers: vec![RetrieverPolicy::Bm25],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::None,
        };

        let corpus = PreparedCorpus {
            documents: &[],
            bm25_index: None,
        };
        let telemetry = crate::system::Telemetry::new();
        let results = service
            .execute(
                &plan,
                "query",
                Some("with intent"),
                &corpus,
                10,
                10,
                0,
                &telemetry,
            )
            .unwrap();

        // Should expand to "query with intent"
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].id, "query with intent");
    }

    #[test]
    fn search_service_uses_hyde_when_llm_reranker_present_and_no_intent() {
        let mut service = SearchService::new();
        service.register_expander(
            QueryExpansionPolicy::Hyde,
            Box::new(LlmExpander::new(Box::new(HydeStrategy))),
        );
        service.register_retriever(Box::new(MockRetriever {
            policy: RetrieverPolicy::Bm25,
        }));
        service.register_fuser(FusionPolicy::Rrf, Box::new(RrfFuser));
        service.register_reranker(RerankingPolicy::Llm, Box::new(NoReranker));

        let plan = SearchPlan {
            name: "test".to_string(),
            query_expansion: QueryExpansionPolicy::None, // Should be overridden by LLM reranker check
            retrievers: vec![RetrieverPolicy::Bm25],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::Llm,
        };

        let corpus = PreparedCorpus {
            documents: &[],
            bm25_index: None,
        };
        let telemetry = crate::system::Telemetry::new();
        let results = service
            .execute(&plan, "query", None, &corpus, 10, 10, 0, &telemetry)
            .unwrap();

        // HydeStrategy (Mock fallback) adds "hypothetical content implementation logic"
        assert!(
            results
                .results
                .iter()
                .any(|c| c.id.contains("hypothetical content"))
        );
    }

    #[test]
    fn search_service_fuses_results_from_multiple_retrievers() {
        let mut service = SearchService::new();
        service.register_expander(QueryExpansionPolicy::None, Box::new(NoExpander));
        service.register_retriever(Box::new(MockRetriever {
            policy: RetrieverPolicy::Bm25,
        }));
        service.register_retriever(Box::new(MockRetriever {
            policy: RetrieverPolicy::Phrase,
        }));
        service.register_fuser(FusionPolicy::Rrf, Box::new(RrfFuser));
        service.register_reranker(RerankingPolicy::None, Box::new(NoReranker));

        let plan = SearchPlan {
            name: "test".to_string(),
            query_expansion: QueryExpansionPolicy::None,
            retrievers: vec![RetrieverPolicy::Bm25, RetrieverPolicy::Phrase],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::None,
        };

        let corpus = PreparedCorpus {
            documents: &[],
            bm25_index: None,
        };
        let telemetry = crate::system::Telemetry::new();
        let results = service
            .execute(&plan, "query", None, &corpus, 10, 10, 0, &telemetry)
            .unwrap();

        // Both retrievers should return "query"
        // RRF should fuse them
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].id, "query");
        // Score for 1 list: 1/(60+1) = 0.01639
        // Score for 2 lists: 1/61 + 1/61 = 0.03278
        assert!(results.results[0].score > 0.03);
    }

    #[test]
    fn strategy_preset_registry_resolves_named_presets_and_hybrid_alias() {
        let registry = StrategyPresetRegistry::default_registry();

        let bm25 = registry.resolve("bm25").unwrap();
        assert_eq!(bm25.name, "bm25");

        let page_index = registry.resolve("page-index").unwrap();
        assert_eq!(page_index.name, "page-index");

        let hybrid = registry.resolve("hybrid").unwrap();
        // hybrid resolves to the champion which is currently page-index-hybrid
        assert_eq!(hybrid.name, "page-index-hybrid");
    }

    #[test]
    fn search_service_shortlist_only_limits_rerank_input() {
        let mut service = SearchService::new();
        service.register_expander(QueryExpansionPolicy::None, Box::new(NoExpander));
        service.register_retriever(Box::new(MockRankedRetriever {
            policy: RetrieverPolicy::Bm25,
            count: 10,
        }));
        service.register_fuser(FusionPolicy::Rrf, Box::new(RrfFuser));
        let (reranker, observed_sizes) = CapturingReranker::new();
        service.register_reranker(RerankingPolicy::Llm, Box::new(reranker));

        let plan = SearchPlan {
            name: "test".to_string(),
            query_expansion: QueryExpansionPolicy::None,
            retrievers: vec![RetrieverPolicy::Bm25],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::Llm,
        };

        let corpus = PreparedCorpus {
            documents: &[],
            bm25_index: None,
        };
        let telemetry = crate::system::Telemetry::new();
        let results = service
            .execute(&plan, "query", None, &corpus, 5, 3, 0, &telemetry)
            .unwrap();

        let observed_sizes = observed_sizes.lock().unwrap();
        assert_eq!(observed_sizes.as_slice(), [3]);
        assert_eq!(results.results.len(), 5);
        assert_eq!(results.results[0].id, "doc02");
        assert_eq!(results.results[1].id, "doc01");
        assert_eq!(results.results[2].id, "doc00");
        assert_eq!(results.results[3].id, "doc03");
        assert_eq!(results.results[4].id, "doc04");
    }

    #[test]
    fn search_service_reranks_shortlist_if_larger_than_limit() {
        let mut service = SearchService::new();
        service.register_expander(QueryExpansionPolicy::None, Box::new(NoExpander));
        service.register_retriever(Box::new(MockRankedRetriever {
            policy: RetrieverPolicy::Bm25,
            count: 10,
        }));
        service.register_fuser(FusionPolicy::Rrf, Box::new(RrfFuser));
        let (reranker, observed_sizes) = CapturingReranker::new();
        service.register_reranker(RerankingPolicy::Llm, Box::new(reranker));

        let plan = SearchPlan {
            name: "test".to_string(),
            query_expansion: QueryExpansionPolicy::None,
            retrievers: vec![RetrieverPolicy::Bm25],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::Llm,
        };

        let corpus = PreparedCorpus {
            documents: &[],
            bm25_index: None,
        };
        let telemetry = crate::system::Telemetry::new();
        let results = service
            .execute(&plan, "query", None, &corpus, 3, 6, 0, &telemetry)
            .unwrap();

        let observed_sizes = observed_sizes.lock().unwrap();
        assert_eq!(observed_sizes.as_slice(), [6]);
        assert_eq!(results.results.len(), 3);
        assert_eq!(results.results[0].id, "doc05");
        assert_eq!(results.results[1].id, "doc04");
        assert_eq!(results.results[2].id, "doc03");
    }
}
