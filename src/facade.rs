use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use anyhow::{Result, bail};

use crate::config::{Config, Ignore};
use crate::dense::DenseModelSpec;
use crate::search::adapters::gemma::GemmaModelSpec;
use crate::search::adapters::qwen::QwenModelSpec;
pub use crate::search::domain::{Conversation, GenerativeModel};
use crate::search::engine::SearchEngine;
use crate::search::{
    Bm25Index, Embedder, FusionPolicy, LatentSearchEmission, LatentSearchHit,
    LocalFileCorpusRepository, ProtocolSearchEmission, QueryEmbeddingCache, RerankingPolicy,
    RetrieverPolicy, SearchControllerAction, SearchControllerDecision, SearchControllerRequest,
    SearchControllerResponse, SearchEmission, SearchEmissionMode, SearchEnvironment, SearchPlan,
    SearchRequest, SearchResponse, SearchServiceBuilder, SearchTrace, SearchTurn,
    SearchTurnRequest, SearchTurnResponse, SearchTurnTrace, StrategyPresetRegistry,
    load_search_corpus, run_search, run_search_with_plan,
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
    Jina,
    Gemma,
}

impl From<Reranking> for RerankingPolicy {
    fn from(value: Reranking) -> Self {
        match value {
            Reranking::None => RerankingPolicy::None,
            Reranking::PositionAware => RerankingPolicy::PositionAware,
            Reranking::Llm => RerankingPolicy::Llm,
            Reranking::Jina => RerankingPolicy::Jina,
            Reranking::Gemma => RerankingPolicy::Gemma,
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

#[derive(Debug, Clone)]
struct BoundedRetainedEvidence {
    retained: Vec<crate::search::RetainedEvidence>,
    pruned: usize,
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
        let gemma_model = self.resolve_gemma_model(&strategy, &input.options);
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
                gemma_model,
                prompts: Some(self.config.prompts.clone()),
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

    pub fn search_turn(&self, request: SearchTurnRequest) -> Result<SearchTurnResponse> {
        let plan = self.resolve_turn_plan(&request)?;
        let search_request = self.build_turn_search_request(&request, &plan);
        let dense_model = search_request.dense_model.clone();
        let embedder = self.resolve_embedder_for_plan(&plan, &dense_model)?;
        let response = run_search_with_plan(
            &plan,
            &search_request,
            self.ignore.as_ref(),
            &LocalFileCorpusRepository,
            embedder,
        )?;

        let mut decisions = vec![
            SearchControllerDecision::new(SearchControllerAction::Retrieve).with_rationale(
                format!("executed {} plan for turn {}", plan.name, request.turn_id),
            ),
        ];

        if !request.retained_evidence.is_empty() {
            decisions.push(
                SearchControllerDecision::new(SearchControllerAction::Retain).with_rationale(
                    format!(
                        "carried {} retained evidence item(s) into this turn",
                        request.retained_evidence.len()
                    ),
                ),
            );
        }

        decisions.push(
            SearchControllerDecision::new(SearchControllerAction::Emit).with_rationale(format!(
                "emitted {} result(s) as {:?}",
                response.results.len(),
                request.emission_mode
            )),
        );
        decisions.push(
            SearchControllerDecision::new(SearchControllerAction::Terminate)
                .with_rationale("completed a direct single-turn search"),
        );

        Ok(self.build_turn_response(
            &request,
            &plan,
            response,
            request.retained_evidence.clone(),
            decisions,
            true,
            Some("single-turn search emitted a terminal response".to_string()),
        ))
    }

    pub fn search_controller(
        &self,
        request: SearchControllerRequest,
    ) -> Result<SearchControllerResponse> {
        if request.turns.is_empty() {
            bail!("controller request requires at least one turn");
        }

        let turn_limit = request.state.turn_limit.min(request.turns.len());
        let session_id = request.session_id.clone().or_else(|| {
            request
                .turns
                .iter()
                .find_map(|turn| turn.session_id.clone())
        });

        if request.state.completed || request.state.next_turn >= turn_limit {
            return Ok(SearchControllerResponse {
                plan: request.plan.clone(),
                state: crate::search::SearchControllerState {
                    completed: true,
                    ..request.state.clone()
                },
                turns: Vec::new(),
                trace: SearchTrace {
                    session_id,
                    turns: Vec::new(),
                    completed: true,
                    termination_reason: Some("controller state already complete".to_string()),
                },
            });
        }

        let first_turn = &request.turns[request.state.next_turn];
        for turn in request.turns.iter().take(turn_limit) {
            if turn.path != first_turn.path {
                bail!("controller turns must share a single corpus path");
            }
        }

        let env_request = self.build_turn_search_request(first_turn, &request.plan);
        let dense_model = env_request.dense_model.clone();
        let embedder = self.resolve_embedder_for_plan(&request.plan, &dense_model)?;
        let corpus = load_search_corpus(
            &first_turn.path,
            self.ignore.as_ref(),
            first_turn.verbose,
            embedder.as_deref(),
            &self.telemetry,
            self.cache_dir.as_deref(),
        )?;
        let index = Bm25Index::build(&corpus.documents);
        let llm_reranker = SearchServiceBuilder::load_llm_reranker(&request.plan, &env_request)?;
        let env = SearchEnvironment::new_with_plan(
            &env_request,
            request.plan.clone(),
            &corpus,
            &index,
            embedder,
            llm_reranker,
        )?;

        let mut state = request.state.clone();
        let mut turn_responses = Vec::new();
        let mut trace_turns = Vec::new();
        let mut previous_turn_id = None;

        while state.next_turn < turn_limit {
            let idx = state.next_turn;
            let mut turn_request = request.turns[idx].clone();
            if turn_request.turn_id == "turn-1" {
                turn_request.turn_id = format!("turn-{}", idx + 1);
            }
            if turn_request.session_id.is_none() {
                turn_request.session_id = session_id.clone();
            }
            if idx > 0 && turn_request.parent_turn_id.is_none() {
                turn_request.parent_turn_id = previous_turn_id.clone();
            }
            turn_request.sequence = idx + 1;
            turn_request.strategy = Some(request.plan.name.clone());
            turn_request.plan = Some(request.plan.clone());
            let carried_evidence = Self::merge_retained_evidence(
                &turn_request.retained_evidence,
                &state.retained_evidence,
                request.retained_evidence_limit,
            );
            turn_request.retained_evidence = carried_evidence.retained.clone();

            let search_request = self.build_turn_search_request(&turn_request, &request.plan);
            let response = env.search(&search_request)?;
            let updated_retained = self.derive_retained_evidence(
                &response,
                &carried_evidence.retained,
                request.retained_evidence_limit,
            );
            let continue_more = idx + 1 < turn_limit;

            let mut decisions = vec![
                SearchControllerDecision::new(SearchControllerAction::Retrieve).with_rationale(
                    format!(
                        "executed turn {} from explicit controller state",
                        turn_request.turn_id
                    ),
                ),
            ];

            if !updated_retained.retained.is_empty() {
                decisions.push(
                    SearchControllerDecision::new(SearchControllerAction::Retain).with_rationale(
                        format!(
                            "retained {} evidence item(s) under the controller budget",
                            updated_retained.retained.len()
                        ),
                    ),
                );
            }

            let pruned = carried_evidence.pruned + updated_retained.pruned;
            if pruned > 0 {
                decisions.push(
                    SearchControllerDecision::new(SearchControllerAction::Prune).with_rationale(
                        format!(
                            "pruned {} evidence item(s) to preserve the controller budget",
                            pruned
                        ),
                    ),
                );
            }

            decisions.push(
                SearchControllerDecision::new(SearchControllerAction::Emit).with_rationale(
                    format!(
                        "emitted {} result(s) for this controller turn",
                        response.results.len()
                    ),
                ),
            );
            decisions.push(
                SearchControllerDecision::new(if continue_more {
                    SearchControllerAction::Continue
                } else {
                    SearchControllerAction::Terminate
                })
                .with_rationale(if continue_more {
                    "continuing to the next planned controller turn"
                } else {
                    "completed the planned controller turn budget"
                }),
            );

            let turn_response = self.build_turn_response(
                &turn_request,
                &request.plan,
                response,
                updated_retained.retained.clone(),
                decisions,
                !continue_more,
                if continue_more {
                    None
                } else {
                    Some("controller executed all planned turns".to_string())
                },
            );

            previous_turn_id = Some(turn_response.turn.turn_id.clone());
            state.retained_evidence = updated_retained.retained;
            state.next_turn = idx + 1;
            state.completed = !continue_more;
            trace_turns.push(turn_response.trace.turns[0].clone());
            turn_responses.push(turn_response);
        }

        Ok(SearchControllerResponse {
            plan: request.plan,
            state: state.clone(),
            turns: turn_responses,
            trace: SearchTrace {
                session_id,
                turns: trace_turns,
                completed: state.completed,
                termination_reason: Some(if state.completed {
                    "controller executed all planned turns".to_string()
                } else {
                    "controller stopped before completing all planned turns".to_string()
                }),
            },
        })
    }

    pub fn generative(&self, options: SearchOptions) -> Result<Arc<dyn GenerativeModel>> {
        let strategy = options
            .strategy
            .clone()
            .unwrap_or_else(|| self.config.search.strategy.clone());
        let rerank_model = self
            .resolve_rerank_model(&strategy, &options)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No generative model (rerank_model) could be resolved from config/options"
                )
            })?;

        println!(
            "[DEBUG] Sift::generative: loading QwenReranker with spec: {:?}",
            rerank_model
        );
        Ok(Arc::new(crate::search::adapters::qwen::QwenReranker::load(
            rerank_model,
        )?))
    }

    fn resolve_turn_plan(&self, request: &SearchTurnRequest) -> Result<SearchPlan> {
        if let Some(plan) = &request.plan {
            return Ok(plan.clone());
        }

        let strategy = request
            .strategy
            .as_deref()
            .unwrap_or(&self.config.search.strategy);
        StrategyPresetRegistry::default_registry().resolve(strategy)
    }

    fn build_turn_search_request(
        &self,
        request: &SearchTurnRequest,
        plan: &SearchPlan,
    ) -> SearchRequest {
        SearchRequest {
            query: request.query.clone(),
            intent: request.intent.clone(),
            path: request.path.clone(),
            strategy: plan.name.clone(),
            limit: request.limit.unwrap_or(self.config.search.limit),
            shortlist: request.shortlist.unwrap_or(self.config.search.shortlist),
            verbose: request.verbose,
            retrievers: None,
            fusion: None,
            reranking: None,
            dense_model: self.default_dense_model(),
            rerank_model: self.resolve_rerank_model_for_plan(plan),
            gemma_model: self.resolve_gemma_model_for_plan(plan),
            query_cache: Some(self.query_cache.clone()),
            cache_dir: self.cache_dir.clone(),
            telemetry: self.telemetry.clone(),
            prompts: Some(self.config.prompts.clone()),
        }
    }

    fn resolve_embedder_for_plan(
        &self,
        plan: &SearchPlan,
        dense_model: &DenseModelSpec,
    ) -> Result<Option<Arc<dyn Embedder>>> {
        if !plan.retrievers.contains(&RetrieverPolicy::Vector) {
            return Ok(None);
        }

        if let Some(embedder) = &self.embedder {
            return Ok(Some(embedder.clone()));
        }

        Ok(Some(
            Arc::new(crate::dense::DenseReranker::load(dense_model.clone())?) as Arc<dyn Embedder>,
        ))
    }

    fn build_turn_response(
        &self,
        request: &SearchTurnRequest,
        plan: &SearchPlan,
        response: SearchResponse,
        retained_evidence: Vec<crate::search::RetainedEvidence>,
        decisions: Vec<SearchControllerDecision>,
        completed: bool,
        termination_reason: Option<String>,
    ) -> SearchTurnResponse {
        let turn = SearchTurn {
            session_id: request.session_id.clone(),
            turn_id: request.turn_id.clone(),
            parent_turn_id: request.parent_turn_id.clone(),
            sequence: request.sequence,
            path: request.path.display().to_string(),
            query: request.query.clone(),
            intent: request.intent.clone(),
            strategy: plan.name.clone(),
            limit: request.limit.unwrap_or(self.config.search.limit),
            shortlist: request.shortlist.unwrap_or(self.config.search.shortlist),
            emission_mode: request.emission_mode,
            result_count: response.results.len(),
            retained_evidence: retained_evidence.clone(),
        };

        let trace_turn = SearchTurnTrace {
            turn_id: turn.turn_id.clone(),
            sequence: turn.sequence,
            query: turn.query.clone(),
            strategy: turn.strategy.clone(),
            emission_mode: turn.emission_mode,
            result_count: turn.result_count,
            retained_evidence: retained_evidence.clone(),
            decisions,
        };

        let trace = SearchTrace {
            session_id: request.session_id.clone(),
            turns: vec![trace_turn],
            completed,
            termination_reason,
        };

        let emission = match request.emission_mode {
            SearchEmissionMode::View => SearchEmission::View(response),
            SearchEmissionMode::Protocol => SearchEmission::Protocol(ProtocolSearchEmission {
                turn_id: request.turn_id.clone(),
                session_id: request.session_id.clone(),
                strategy: plan.name.clone(),
                root: response.root.clone(),
                hits: response.results.clone(),
            }),
            SearchEmissionMode::Latent => SearchEmission::Latent(LatentSearchEmission {
                turn_id: request.turn_id.clone(),
                session_id: request.session_id.clone(),
                feature_space: "ranking-score".to_string(),
                hits: response
                    .results
                    .iter()
                    .map(|hit| LatentSearchHit {
                        path: hit.path.clone(),
                        score: hit.score,
                        confidence: hit.confidence,
                        location: hit.location.clone(),
                    })
                    .collect(),
            }),
        };

        SearchTurnResponse {
            turn,
            trace,
            emission,
        }
    }

    fn merge_retained_evidence(
        primary: &[crate::search::RetainedEvidence],
        secondary: &[crate::search::RetainedEvidence],
        limit: usize,
    ) -> BoundedRetainedEvidence {
        let mut unique = Vec::new();
        let mut seen = HashSet::new();

        for evidence in primary.iter().chain(secondary.iter()) {
            let key = format!(
                "{}\n{}\n{}",
                evidence.path,
                evidence.location.as_deref().unwrap_or(""),
                evidence.snippet.as_deref().unwrap_or("")
            );
            if seen.insert(key) {
                unique.push(evidence.clone());
            }
        }

        let pruned = unique.len().saturating_sub(limit);

        BoundedRetainedEvidence {
            retained: unique.into_iter().take(limit).collect(),
            pruned,
        }
    }

    fn derive_retained_evidence(
        &self,
        response: &SearchResponse,
        prior: &[crate::search::RetainedEvidence],
        limit: usize,
    ) -> BoundedRetainedEvidence {
        let fresh: Vec<_> = response
            .results
            .iter()
            .take(limit)
            .map(|hit| {
                let mut evidence = crate::search::RetainedEvidence::new(hit.path.clone());
                if let Some(location) = &hit.location {
                    evidence = evidence.with_location(location.clone());
                }
                if !hit.snippet.is_empty() {
                    evidence = evidence.with_snippet(hit.snippet.clone());
                }
                evidence.with_rationale(format!(
                    "retained rank {} from {}",
                    hit.rank, response.strategy
                ))
            })
            .collect();

        Self::merge_retained_evidence(&fresh, prior, limit)
    }

    fn default_dense_model(&self) -> DenseModelSpec {
        DenseModelSpec::with_overrides(
            Some(self.config.embedding.model_id.clone()),
            Some(self.config.embedding.model_revision.clone()),
            Some(self.config.embedding.max_length),
        )
    }

    fn default_rerank_model(&self) -> QwenModelSpec {
        QwenModelSpec {
            model_id: self.config.rerank.model_id.clone(),
            revision: self.config.rerank.model_revision.clone(),
            max_length: self.config.rerank.max_length,
        }
    }

    fn default_gemma_model(&self) -> GemmaModelSpec {
        GemmaModelSpec {
            model_id: self.config.gemma.model_id.clone(),
            revision: self.config.gemma.model_revision.clone(),
            max_length: self.config.gemma.max_length,
        }
    }

    fn resolve_rerank_model_for_plan(&self, plan: &SearchPlan) -> Option<QwenModelSpec> {
        if plan.reranking == RerankingPolicy::Llm {
            return Some(self.default_rerank_model());
        }

        None
    }

    fn resolve_gemma_model_for_plan(&self, plan: &SearchPlan) -> Option<GemmaModelSpec> {
        if plan.reranking == RerankingPolicy::Gemma {
            return Some(self.default_gemma_model());
        }

        None
    }

    fn resolve_embedder(
        &self,
        strategy: &str,
        options: &SearchOptions,
        dense_model: &DenseModelSpec,
    ) -> Result<Option<Arc<dyn Embedder>>> {
        let plan = resolve_plan(strategy, options)?;
        self.resolve_embedder_for_plan(&plan, dense_model)
    }

    fn resolve_rerank_model(
        &self,
        strategy: &str,
        options: &SearchOptions,
    ) -> Option<QwenModelSpec> {
        if let Some(spec) = &options.rerank_model {
            return Some(spec.clone());
        }

        let plan = match resolve_plan(strategy, options) {
            Ok(plan) => plan,
            Err(_) => return None,
        };

        self.resolve_rerank_model_for_plan(&plan)
    }

    fn resolve_gemma_model(
        &self,
        strategy: &str,
        options: &SearchOptions,
    ) -> Option<GemmaModelSpec> {
        if let Some(spec) = &options.gemma_model {
            return Some(spec.clone());
        }

        let plan = match resolve_plan(strategy, options) {
            Ok(plan) => plan,
            Err(_) => return None,
        };

        self.resolve_gemma_model_for_plan(&plan)
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
    gemma_model: Option<GemmaModelSpec>,
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

    pub fn with_gemma_model(mut self, gemma_model: GemmaModelSpec) -> Self {
        self.gemma_model = Some(gemma_model);
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
