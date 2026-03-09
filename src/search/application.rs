use anyhow::{Result, anyhow};
use std::collections::HashMap;

use super::adapters::*;
use super::corpus::load_search_corpus;
use super::domain::*;
use crate::config::Ignore;
use crate::dense::DenseReranker;

pub struct SearchService {
    retrievers: HashMap<RetrieverPolicy, Box<dyn Retriever>>,
    fusers: HashMap<FusionPolicy, Box<dyn Fuser>>,
    expanders: HashMap<QueryExpansionPolicy, Box<dyn Expander>>,
    rerankers: HashMap<RerankingPolicy, Box<dyn Reranker>>,
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
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::PositionAware,
            },
        );

        // page-index-hybrid (champion, lexical + vector)
        registry.register(
            "page-index-hybrid",
            SearchPlan {
                name: "page-index-hybrid".to_string(),
                query_expansion: QueryExpansionPolicy::None,
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
        self.rerankers.insert(policy, reranker);
    }

    pub fn execute(
        &self,
        plan: &SearchPlan,
        query: &str,
        corpus: &PreparedCorpus,
        limit: usize,
        _verbose: u8,
        telemetry: &crate::system::Telemetry,
    ) -> Result<CandidateList> {
        let start = std::time::Instant::now();
        tracing::info!("executing strategy: {} for query: '{}'", plan.name, query);

        // 1. Query Expansion
        let expand_start = std::time::Instant::now();
        let expander = self
            .expanders
            .get(&plan.query_expansion)
            .ok_or_else(|| anyhow!("expander not found for policy: {:?}", plan.query_expansion))?;
        let query_variants = expander.expand(query);
        tracing::info!("expanded query into {} variants in {:.2?}", query_variants.len(), expand_start.elapsed());
        tracing::debug!("variants: {:?}", query_variants.iter().map(|v| &v.text).collect::<Vec<_>>());

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
            let list = retriever.retrieve(&query_variants, corpus, limit, 0)?;
            tracing::info!("{:?}: found {} candidates in {:.2?}", policy, list.results.len(), retrieve_start.elapsed());
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
        let fused = fuser.fuse(&candidate_lists, limit, 0)?;
        tracing::info!("fused into {} candidates in {:.2?}", fused.results.len(), fuse_start.elapsed());

        // 4. Reranking
        let rerank_start = std::time::Instant::now();
        let reranker = self
            .rerankers
            .get(&plan.reranking)
            .ok_or_else(|| anyhow!("reranker not found for policy: {:?}", plan.reranking))?;
        let final_list = reranker.rerank(query, fused, limit)?;
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
        dense: Option<std::sync::Arc<DenseReranker>>,
    ) -> Result<Self> {
        let mut service = SearchService::new();
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

        service.register_retriever(Box::new(Bm25Retriever));
        service.register_retriever(Box::new(PhraseRetriever));
        if let Some(dense) = dense {
            service.register_retriever(Box::new(SegmentVectorRetriever { dense }));
        }

        service.register_fuser(FusionPolicy::Rrf, Box::new(RrfFuser));
        service.register_expander(QueryExpansionPolicy::None, Box::new(NoExpander));
        service.register_expander(QueryExpansionPolicy::Synonym, Box::new(SynonymExpander));
        service.register_reranker(RerankingPolicy::None, Box::new(NoReranker));
        service.register_reranker(RerankingPolicy::PositionAware, Box::new(PositionAwareReranker));
        service.register_reranker(RerankingPolicy::Llm, Box::new(MockLlmReranker));

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

    pub fn search(&self, query: &str, limit: usize, verbose: u8) -> Result<SearchResponse> {
        let candidates = self.service.execute(&self.plan, query, &self.prepared, limit, verbose, &self.telemetry)?;

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
            telemetry: Some(SearchTelemetry {
                heuristic_hit_rate: self.telemetry.heuristic_hit_rate(),
                blob_hit_rate: self.telemetry.blob_hit_rate(),
                embedding_hit_rate: self.telemetry.embedding_hit_rate(),
            }),
        })
    }
}

pub fn run_search(request: &SearchRequest, ignore: Option<&Ignore>) -> Result<SearchResponse> {
    let mut service = SearchService::new();
    let verbose = request.verbose;

    // Register adapters
    service.register_retriever(Box::new(Bm25Retriever));
    service.register_retriever(Box::new(PhraseRetriever));

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
    
    // Determine if we need to pass the dense model for embedding during load
    let mut dense_for_load = None;
    if plan.retrievers.contains(&RetrieverPolicy::Vector) {
        let model_start = std::time::Instant::now();
        let dense = std::sync::Arc::new(DenseReranker::load(request.dense_model.clone())?);
        tracing::info!("dense model loaded in {:.2?}", model_start.elapsed());
        dense_for_load = Some(dense);
    }

    let corpus = load_search_corpus(&request.path, ignore, verbose, dense_for_load.as_deref(), &request.telemetry)?;
    tracing::info!("corpus loaded ({} files) in {:.2?}", corpus.indexed_files, corpus_start.elapsed());

    if let Some(dense) = dense_for_load {
        service.register_retriever(Box::new(SegmentVectorRetriever { dense }));
    }

    service.register_fuser(FusionPolicy::Rrf, Box::new(RrfFuser));
    service.register_expander(QueryExpansionPolicy::None, Box::new(NoExpander));
    service.register_expander(QueryExpansionPolicy::Synonym, Box::new(SynonymExpander));
    service.register_reranker(RerankingPolicy::None, Box::new(NoReranker));
    service.register_reranker(RerankingPolicy::PositionAware, Box::new(PositionAwareReranker));
    service.register_reranker(RerankingPolicy::Llm, Box::new(MockLlmReranker));

    let index_start = std::time::Instant::now();
    let index = Bm25Index::build(&corpus.documents);
    tracing::info!("built bm25 index in {:.2?}", index_start.elapsed());
    let prepared = PreparedCorpus {
        documents: &corpus.documents,
        bm25_index: Some(&index),
    };

    let candidates = service.execute(&plan, &request.query, &prepared, request.limit, verbose, &request.telemetry)?;

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
        telemetry: Some(SearchTelemetry {
            heuristic_hit_rate: request.telemetry.heuristic_hit_rate(),
            blob_hit_rate: request.telemetry.blob_hit_rate(),
            embedding_hit_rate: request.telemetry.embedding_hit_rate(),
        }),
    })
}

fn resolve_snippet_from_candidate(
    corpus: &LoadedCorpus,
    candidate: &Candidate,
    query: &str,
) -> String {
    if let Some(snippet) = candidate.snippet.as_deref() {
        return build_snippet(snippet, query);
    }

    build_snippet(
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
        let results = service.execute(&plan, "search", &corpus, 10, 0).unwrap();

        // "search" expansion with SynonymExpander gives "search" and "retrieval"
        // MockRetriever returns them as candidates
        assert!(results.results.iter().any(|c| c.id == "search"));
        assert!(results.results.iter().any(|c| c.id == "retrieval"));
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
        let results = service.execute(&plan, "query", &corpus, 10, 0).unwrap();

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
        // hybrid resolves to the champion which is currently page-index
        assert_eq!(hybrid.name, "page-index");
    }
}
