use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use anyhow::Result;

use crate::config::{Config, Ignore};
use crate::dense::DenseModelSpec;
use crate::search::adapters::qwen::QwenModelSpec;
use crate::search::{
    Embedder, FusionPolicy, LocalFileCorpusRepository, QueryEmbeddingCache, RerankingPolicy,
    RetrieverPolicy, SearchRequest, SearchResponse, StrategyPresetRegistry, run_search,
};
use crate::system::Telemetry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retriever {
    Bm25,
    Phrase,
    Vector,
}

impl From<Retriever> for RetrieverPolicy {
    fn from(value: Retriever) -> Self {
        match value {
            Retriever::Bm25 => RetrieverPolicy::Bm25,
            Retriever::Phrase => RetrieverPolicy::Phrase,
            Retriever::Vector => RetrieverPolicy::Vector,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fusion {
    Rrf,
}

impl From<Fusion> for FusionPolicy {
    fn from(value: Fusion) -> Self {
        match value {
            Fusion::Rrf => FusionPolicy::Rrf,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reranking {
    None,
    PositionAware,
    Llm,
}

impl From<Reranking> for RerankingPolicy {
    fn from(value: Reranking) -> Self {
        match value {
            Reranking::None => RerankingPolicy::None,
            Reranking::PositionAware => RerankingPolicy::PositionAware,
            Reranking::Llm => RerankingPolicy::Llm,
        }
    }
}

pub struct SiftBuilder {
    config: Config,
    ignore: Option<Ignore>,
    telemetry: Arc<Telemetry>,
    query_cache: QueryEmbeddingCache,
    cache_dir: Option<PathBuf>,
    embedder: Option<Arc<dyn Embedder>>,
}

impl Default for SiftBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SiftBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            ignore: Some(Ignore::load()),
            telemetry: Arc::new(Telemetry::new()),
            query_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_dir: None,
            embedder: None,
        }
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn with_ignore(mut self, ignore: Ignore) -> Self {
        self.ignore = Some(ignore);
        self
    }

    pub fn without_ignore(mut self) -> Self {
        self.ignore = None;
        self
    }

    pub fn with_telemetry(mut self, telemetry: Arc<Telemetry>) -> Self {
        self.telemetry = telemetry;
        self
    }

    pub fn with_query_cache(mut self, query_cache: QueryEmbeddingCache) -> Self {
        self.query_cache = query_cache;
        self
    }

    pub fn with_cache_dir(mut self, cache_dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(cache_dir.into());
        self
    }

    pub fn with_embedder(mut self, embedder: Arc<dyn Embedder>) -> Self {
        self.embedder = Some(embedder);
        self
    }

    pub fn build(self) -> Sift {
        Sift {
            config: self.config,
            ignore: self.ignore,
            telemetry: self.telemetry,
            query_cache: self.query_cache,
            cache_dir: self.cache_dir,
            embedder: self.embedder,
        }
    }
}

pub struct Sift {
    config: Config,
    ignore: Option<Ignore>,
    telemetry: Arc<Telemetry>,
    query_cache: QueryEmbeddingCache,
    cache_dir: Option<PathBuf>,
    embedder: Option<Arc<dyn Embedder>>,
}

impl Sift {
    pub fn builder() -> SiftBuilder {
        SiftBuilder::new()
    }

    pub fn search(&self, input: SearchInput) -> Result<SearchResponse> {
        let strategy = input
            .options
            .strategy
            .clone()
            .unwrap_or_else(|| self.config.search.strategy.clone());
        let intent = input
            .intent
            .as_ref()
            .or(input.options.intent.as_ref())
            .cloned();
        let limit = input.options.limit.unwrap_or(self.config.search.limit);
        let shortlist = input
            .options
            .shortlist
            .unwrap_or(self.config.search.shortlist);
        let dense_model = input.options.dense_model.clone().unwrap_or_else(|| {
            DenseModelSpec::with_overrides(
                Some(self.config.embedding.model_id.clone()),
                Some(self.config.embedding.model_revision.clone()),
                Some(self.config.embedding.max_length),
            )
        });
        let rerank_model = self.resolve_rerank_model(&strategy, &input.options);
        let embedder = self.resolve_embedder(&strategy, &input.options, &dense_model)?;

        run_search(
            &SearchRequest {
                strategy,
                query: input.query,
                intent,
                path: input.path,
                limit,
                shortlist,
                dense_model,
                rerank_model,
                verbose: input.options.verbose,
                retrievers: input
                    .options
                    .retrievers
                    .map(|retrievers| retrievers.into_iter().map(Into::into).collect()),
                fusion: input.options.fusion.map(Into::into),
                reranking: input.options.reranking.map(Into::into),
                telemetry: self.telemetry.clone(),
                cache_dir: input.options.cache_dir.or_else(|| self.cache_dir.clone()),
                query_cache: Some(self.query_cache.clone()),
            },
            self.ignore.as_ref(),
            &LocalFileCorpusRepository,
            embedder,
        )
    }

    fn resolve_embedder(
        &self,
        strategy: &str,
        options: &SearchOptions,
        dense_model: &DenseModelSpec,
    ) -> Result<Option<Arc<dyn Embedder>>> {
        let plan = resolve_plan(strategy, options)?;
        let needs_vector = plan.retrievers.contains(&RetrieverPolicy::Vector);

        if !needs_vector {
            return Ok(None);
        }

        if let Some(embedder) = &self.embedder {
            return Ok(Some(embedder.clone()));
        }

        Ok(Some(
            Arc::new(crate::dense::DenseReranker::load(dense_model.clone())?) as Arc<dyn Embedder>,
        ))
    }

    fn resolve_rerank_model(
        &self,
        strategy: &str,
        options: &SearchOptions,
    ) -> Option<QwenModelSpec> {
        let plan = match resolve_plan(strategy, options) {
            Ok(plan) => plan,
            Err(_) => return options.rerank_model.clone(),
        };

        if options.rerank_model.is_some() || plan.reranking == RerankingPolicy::Llm {
            return options.rerank_model.clone().or_else(|| {
                Some(QwenModelSpec {
                    model_id: self.config.rerank.model_id.clone(),
                    revision: self.config.rerank.model_revision.clone(),
                    max_length: self.config.rerank.max_length,
                })
            });
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct SearchInput {
    path: PathBuf,
    query: String,
    intent: Option<String>,
    options: SearchOptions,
}

impl SearchInput {
    pub fn new(path: impl AsRef<Path>, query: impl Into<String>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            query: query.into(),
            intent: None,
            options: SearchOptions::default(),
        }
    }

    pub fn with_intent(mut self, intent: impl Into<String>) -> Self {
        self.intent = Some(intent.into());
        self
    }

    pub fn with_options(mut self, options: SearchOptions) -> Self {
        self.options = options;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    strategy: Option<String>,
    intent: Option<String>,
    limit: Option<usize>,
    shortlist: Option<usize>,
    dense_model: Option<DenseModelSpec>,
    rerank_model: Option<QwenModelSpec>,
    verbose: u8,
    retrievers: Option<Vec<Retriever>>,
    fusion: Option<Fusion>,
    reranking: Option<Reranking>,
    cache_dir: Option<PathBuf>,
}

impl SearchOptions {
    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.strategy = Some(strategy.into());
        self
    }

    pub fn with_intent(mut self, intent: impl Into<String>) -> Self {
        self.intent = Some(intent.into());
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_shortlist(mut self, shortlist: usize) -> Self {
        self.shortlist = Some(shortlist);
        self
    }

    pub fn with_dense_model(mut self, dense_model: DenseModelSpec) -> Self {
        self.dense_model = Some(dense_model);
        self
    }

    pub fn with_rerank_model(mut self, rerank_model: QwenModelSpec) -> Self {
        self.rerank_model = Some(rerank_model);
        self
    }

    pub fn with_verbose(mut self, verbose: u8) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_retrievers(mut self, retrievers: Vec<Retriever>) -> Self {
        self.retrievers = Some(retrievers);
        self
    }

    pub fn with_fusion(mut self, fusion: Fusion) -> Self {
        self.fusion = Some(fusion);
        self
    }

    pub fn with_reranking(mut self, reranking: Reranking) -> Self {
        self.reranking = Some(reranking);
        self
    }

    pub fn with_cache_dir(mut self, cache_dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(cache_dir.into());
        self
    }
}

fn resolve_plan(strategy: &str, options: &SearchOptions) -> Result<crate::search::SearchPlan> {
    let registry = StrategyPresetRegistry::default_registry();
    let mut plan = registry.resolve(strategy)?;

    if let Some(retrievers) = &options.retrievers {
        plan.retrievers = retrievers.iter().copied().map(Into::into).collect();
    }
    if let Some(fusion) = options.fusion {
        plan.fusion = fusion.into();
    }
    if let Some(reranking) = options.reranking {
        plan.reranking = reranking.into();
    }

    Ok(plan)
}
