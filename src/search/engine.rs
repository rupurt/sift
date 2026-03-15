use super::adapters::*;
use super::application::{SearchService, StrategyPresetRegistry, resolve_snippet_from_candidate};
use super::domain::{
    Bm25Index, CachedEmbedder, Embedder, FusionPolicy, GenerativeModel, LoadedCorpus,
    PreparedCorpus, QueryEmbeddingCache, QueryExpansionPolicy, Reranker, RerankingPolicy,
    SearchHit, SearchPlan, SearchRequest, SearchResponse,
};
use anyhow::Result;
use std::sync::Arc;

/// Abstract persistence layer for search artifacts.
pub trait SearchStorage: Send + Sync {
    fn corpus(&self) -> &LoadedCorpus;
    fn bm25_index(&self) -> &Bm25Index;
}

/// Abstract representation of a search strategy.
/// Initially this wraps the SearchPlan, but will evolve into a Graph IR.
pub trait SearchIR: Send + Sync {
    fn plan(&self) -> &SearchPlan;
}

/// Abstract execution runtime for a search graph.
pub trait SearchExecution: Send + Sync {
    fn execute(
        &self,
        ir: &dyn SearchIR,
        storage: &dyn SearchStorage,
        request: &SearchRequest,
    ) -> Result<SearchResponse>;
}

/// The unified search engine orchestrator.
pub trait SearchEngine: Send + Sync {
    fn search(&self, request: &SearchRequest) -> Result<SearchResponse>;
    fn as_any(&self) -> &dyn std::any::Any;
}

/// A concrete implementation of SearchEngine that ties the traits together.
pub struct GenericEngine<IR, Exec, Storage>
where
    IR: SearchIR,
    Exec: SearchExecution,
    Storage: SearchStorage,
{
    pub ir: IR,
    pub execution: Exec,
    pub storage: Storage,
}

impl<IR, Exec, Storage> SearchEngine for GenericEngine<IR, Exec, Storage>
where
    IR: SearchIR + 'static,
    Exec: SearchExecution + 'static,
    Storage: SearchStorage + 'static,
{
    fn search(&self, request: &SearchRequest) -> Result<SearchResponse> {
        self.execution.execute(&self.ir, &self.storage, request)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct EngineFactory {
    pub registry: StrategyPresetRegistry,
}

impl Default for EngineFactory {
    fn default() -> Self {
        Self {
            registry: StrategyPresetRegistry::default_registry(),
        }
    }
}

impl EngineFactory {
    pub fn build(
        &self,
        strategy: &str,
        storage: LocalCorpusStorage,
        embedder: Option<Arc<dyn Embedder>>,
        query_cache: Option<QueryEmbeddingCache>,
        llm_reranker: Option<Arc<dyn Reranker>>,
    ) -> Result<Box<dyn SearchEngine>> {
        let plan = self.registry.resolve(strategy)?;

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

        service.register_retriever(Box::new(Bm25Retriever));
        service.register_retriever(Box::new(PhraseRetriever));

        if let Some(e) = embedder {
            let final_embedder = if let Some(cache) = query_cache {
                Arc::new(CachedEmbedder { inner: e, cache }) as Arc<dyn Embedder>
            } else {
                e
            };
            service.register_retriever(Box::new(SegmentVectorRetriever {
                embedder: final_embedder,
            }));
        }

        let ir = PresetIR { plan };
        let execution = PipelineExecution { service };

        Ok(Box::new(GenericEngine {
            ir,
            execution,
            storage,
        }))
    }
}

// --- STANDARD ADAPTERS ---

/// Standard storage adapter for local in-memory corpora.
#[derive(Clone)]
pub struct LocalCorpusStorage {
    pub corpus: Arc<LoadedCorpus>,
    pub index: Arc<Bm25Index>,
}

impl SearchStorage for LocalCorpusStorage {
    fn corpus(&self) -> &LoadedCorpus {
        &self.corpus
    }
    fn bm25_index(&self) -> &Bm25Index {
        &self.index
    }
}

/// Standard IR adapter for named presets.
pub struct PresetIR {
    pub plan: SearchPlan,
}

impl SearchIR for PresetIR {
    fn plan(&self) -> &SearchPlan {
        &self.plan
    }
}

/// Standard execution adapter for the multi-stage search pipeline.
pub struct PipelineExecution {
    pub service: SearchService,
}

impl SearchExecution for PipelineExecution {
    fn execute(
        &self,
        ir: &dyn SearchIR,
        storage: &dyn SearchStorage,
        request: &SearchRequest,
    ) -> Result<SearchResponse> {
        let plan = ir.plan();
        let corpus = storage.corpus();
        let index = storage.bm25_index();
        let prepared = PreparedCorpus {
            documents: &corpus.documents,
            bm25_index: Some(index),
        };

        let candidates = self.service.execute(
            plan,
            &request.query,
            request.intent.as_deref(),
            &prepared,
            request.limit,
            request.shortlist,
            request.verbose,
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
                    snippet: resolve_snippet_from_candidate(corpus, &result, &request.query),
                }
            })
            .collect();

        Ok(SearchResponse {
            strategy: plan.name.clone(),
            root: request.path.display().to_string(),
            indexed_files: corpus.indexed_files,
            skipped_files: corpus.skipped_files,
            results,
        })
    }
}
