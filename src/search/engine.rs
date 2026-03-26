use super::adapters::*;
use super::application::{SearchService, resolve_snippet_from_candidate};
use super::domain::{
    Bm25Index, Embedder, FusionPolicy, GenerativeModel, LoadedCorpus, PreparedCorpus,
    QueryEmbeddingCache, QueryExpansionPolicy, Reranker, RerankingPolicy, SearchHit, SearchPlan,
    SearchRequest, SearchResponse, StrategyPresetRegistry,
};
use anyhow::Result;
use std::sync::Arc;

/// Abstract persistence layer for search artifacts.
pub trait SearchStorage: Send + Sync {
    fn corpus(&self) -> &LoadedCorpus;
    fn bm25_index(&self) -> &Bm25Index;
}

/// Abstract representation of a search strategy.
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

#[derive(Clone)]
pub struct SearchEnvironment {
    pub ir: PresetIR,
    pub execution: PipelineExecution,
    pub storage: LocalCorpusStorage,
    pub telemetry: Arc<crate::system::Telemetry>,
}

impl SearchEngine for SearchEnvironment {
    fn search(&self, request: &SearchRequest) -> Result<SearchResponse> {
        self.execution.execute(&self.ir, &self.storage, request)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SearchEnvironment {
    pub fn new(
        request: &SearchRequest,
        corpus: &LoadedCorpus,
        index: &Bm25Index,
        embedder: Option<Arc<dyn Embedder>>,
        llm_reranker: Option<Arc<dyn Reranker>>,
    ) -> Result<Self> {
        let factory = EngineFactory::default();
        let engine = factory.build(
            &request.strategy,
            LocalCorpusStorage {
                corpus: Arc::new(corpus.clone()),
                index: Arc::new(index.clone()),
            },
            embedder,
            request.query_cache.clone(),
            llm_reranker,
        )?;

        // Downcast to concrete type
        let boxed_any = engine.as_any();
        if let Some(env) = boxed_any
            .downcast_ref::<GenericEngine<PresetIR, PipelineExecution, LocalCorpusStorage>>()
        {
            Ok(SearchEnvironment {
                ir: env.ir.clone(),
                execution: env.execution.clone(),
                storage: env.storage.clone(),
                telemetry: request.telemetry.clone(),
            })
        } else {
            anyhow::bail!("failed to downcast search engine to SearchEnvironment")
        }
    }
}

/// A concrete implementation of SearchEngine that ties the traits together.
#[derive(Clone)]
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
    IR: SearchIR + 'static + Clone,
    Exec: SearchExecution + 'static + Clone,
    Storage: SearchStorage + 'static + Clone,
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

        let mut hyde = LlmExpander::new(Box::new(HydeStrategy {
            custom_prompt: None,
        }));
        let mut splade = LlmExpander::new(Box::new(SpladeStrategy {
            custom_prompt: None,
        }));
        let mut classified = LlmExpander::new(Box::new(ClassifiedStrategy {
            custom_prompt: None,
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

        let ir = PresetIR { plan };
        let execution = PipelineExecution {
            service: Arc::new(service),
        };

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
#[derive(Clone)]
pub struct PresetIR {
    pub plan: SearchPlan,
}

impl SearchIR for PresetIR {
    fn plan(&self) -> &SearchPlan {
        &self.plan
    }
}

/// Standard execution adapter for the multi-stage search pipeline.
#[derive(Clone)]
pub struct PipelineExecution {
    pub service: Arc<SearchService>,
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

        let results = self.service.execute(plan, request, &prepared)?;

        let hits = results
            .results
            .into_iter()
            .enumerate()
            .map(|(idx, res)| {
                let mut path = res.path.display().to_string();
                if path.starts_with("./") {
                    path = path.chars().skip(2).collect();
                }
                SearchHit {
                    path,
                    rank: idx + 1,
                    score: res.score,
                    confidence: plan.categorize_score(res.score),
                    location: res.snippet_location.clone(),
                    snippet: resolve_snippet_from_candidate(corpus, &res, &request.query),
                }
            })
            .collect();

        Ok(SearchResponse {
            strategy: request.strategy.clone(),
            root: request.path.display().to_string(),
            indexed_files: corpus.indexed_files,
            skipped_files: corpus.skipped_files,
            results: hits,
        })
    }
}
