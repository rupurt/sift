use anyhow::{Result, anyhow};
use std::collections::HashMap;

use super::adapters::*;
use super::corpus::load_search_corpus;
use super::domain::*;
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
        let mut registry = Self::new("page-index");

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

        // hybrid preset (as an explicit preset)
        registry.register(
            "legacy-hybrid",
            SearchPlan {
                name: "legacy-hybrid".to_string(),
                query_expansion: QueryExpansionPolicy::None,
                retrievers: vec![RetrieverPolicy::Bm25, RetrieverPolicy::SegmentVector],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::None,
            },
        );

        // page-index inspired preset
        registry.register(
            "page-index",
            SearchPlan {
                name: "page-index".to_string(),
                query_expansion: QueryExpansionPolicy::None,
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                    RetrieverPolicy::SegmentVector,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::None,
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
    ) -> Result<CandidateList> {
        // 1. Query Expansion
        let expander = self
            .expanders
            .get(&plan.query_expansion)
            .ok_or_else(|| anyhow!("expander not found for policy: {:?}", plan.query_expansion))?;
        let query_variants = expander.expand(query);

        // 2. Retrieval
        let mut candidate_lists = Vec::new();
        for policy in &plan.retrievers {
            let retriever = self
                .retrievers
                .get(policy)
                .ok_or_else(|| anyhow!("retriever not found for policy: {:?}", policy))?;
            candidate_lists.push(retriever.retrieve(&query_variants, corpus, limit)?);
        }

        if candidate_lists.is_empty() {
            return Ok(CandidateList {
                results: Vec::new(),
            });
        }

        // 3. Fusion
        let fuser = self
            .fusers
            .get(&plan.fusion)
            .ok_or_else(|| anyhow!("fuser not found for policy: {:?}", plan.fusion))?;
        let fused = fuser.fuse(&candidate_lists, limit)?;

        // 4. Reranking
        let reranker = self
            .rerankers
            .get(&plan.reranking)
            .ok_or_else(|| anyhow!("reranker not found for policy: {:?}", plan.reranking))?;
        let final_list = reranker.rerank(query, fused, limit)?;

        Ok(final_list)
    }
}

pub fn run_search(request: &SearchRequest) -> Result<SearchResponse> {
    let mut service = SearchService::new();

    // Register adapters
    service.register_retriever(Box::new(Bm25Retriever));
    service.register_retriever(Box::new(PhraseRetriever));

    // For SegmentVectorRetriever, we need to load the model if the plan requires it
    let registry = StrategyPresetRegistry::default_registry();
    let plan = registry.resolve(&request.strategy)?;

    if plan.retrievers.contains(&RetrieverPolicy::SegmentVector) {
        let dense = DenseReranker::load(request.dense_model.clone())?;
        service.register_retriever(Box::new(SegmentVectorRetriever { dense }));
    }

    service.register_fuser(FusionPolicy::Rrf, Box::new(RrfFuser));
    service.register_expander(QueryExpansionPolicy::None, Box::new(NoExpander));
    service.register_reranker(RerankingPolicy::None, Box::new(NoReranker));

    let corpus = load_search_corpus(&request.path)?;
    let index = Bm25Index::build(&corpus.documents);
    let prepared = PreparedCorpus {
        documents: &corpus.documents,
        bm25_index: Some(&index),
    };

    let candidates = service.execute(&plan, &request.query, &prepared, request.limit)?;

    let results = candidates
        .results
        .into_iter()
        .enumerate()
        .map(|(index, result)| SearchHit {
            path: result.path.display().to_string(),
            rank: index + 1,
            score: result.score,
            snippet: resolve_snippet_from_candidate(&corpus, &result),
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

fn resolve_snippet_from_candidate(corpus: &LoadedCorpus, candidate: &Candidate) -> String {
    if let Some(snippet) = candidate.snippet.as_deref() {
        return build_snippet(snippet);
    }

    build_snippet(
        corpus
            .document_by_id(&candidate.id)
            .map(Document::text)
            .unwrap_or_default(),
    )
}
