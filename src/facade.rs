use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow, bail};

use crate::config::{Config, Ignore};
use crate::dense::DenseModelSpec;
use crate::search::adapters::gemma::GemmaModelSpec;
use crate::search::adapters::qwen::QwenModelSpec;
pub use crate::search::domain::{Conversation, GenerativeModel};
use crate::search::engine::SearchEngine;
use crate::search::{
    AutonomousGraphBranchState, AutonomousGraphBranchStatus, AutonomousGraphEpisodeState,
    AutonomousGraphNode, AutonomousPlanner, AutonomousPlannerAction, AutonomousPlannerStopReason,
    AutonomousPlannerStrategyKind, AutonomousPlannerTrace, AutonomousSearchMode,
    AutonomousSearchRequest, AutonomousSearchResponse, ContextAssemblyBudget,
    ContextAssemblyRequest, ContextAssemblyResponse, Embedder, FusionPolicy,
    HeuristicAutonomousPlanner, LatentSearchEmission, LatentSearchHit, LocalFileCorpusRepository,
    ModelDrivenAutonomousPlanner, ProtocolSearchEmission, QueryEmbeddingCache, RerankingPolicy,
    RetrieverPolicy, SearchControllerAction, SearchControllerDecision, SearchControllerRequest,
    SearchControllerResponse, SearchControllerState, SearchEmission, SearchEmissionMode,
    SearchEnvironment, SearchPhase, SearchPlan, SearchProgress, SearchRequest, SearchResponse,
    SearchServiceBuilder, SearchTelemetry, SearchTrace, SearchTurn, SearchTurnRequest,
    SearchTurnResponse, SearchTurnTrace, StrategyPresetRegistry, replay_graph_decision,
    replay_graph_trace, run_search_with_plan, run_search_with_plan_and_progress,
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
    generative_model: Option<Arc<dyn GenerativeModel>>,
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
            generative_model: None,
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

    pub fn with_generative_model(mut self, generative_model: Arc<dyn GenerativeModel>) -> Self {
        self.generative_model = Some(generative_model);
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
            generative_model: self.generative_model,
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
    generative_model: Option<Arc<dyn GenerativeModel>>,
}

#[derive(Debug, Clone)]
struct BoundedRetainedArtifacts {
    retained: Vec<crate::search::RetainedArtifact>,
    pruned: usize,
}

#[derive(Debug, Clone)]
struct GraphSearchTurnContext {
    sequence: usize,
    continue_more: bool,
    previous_turn_id: Option<String>,
}

impl Sift {
    pub fn builder() -> SiftBuilder {
        SiftBuilder::new()
    }

    fn estimate_remaining(start: Instant, processed: usize, total: usize) -> Option<Duration> {
        if processed == 0 || total == 0 {
            return None;
        }
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            return None;
        }
        let remaining = total.saturating_sub(processed);
        if remaining == 0 {
            return Some(Duration::from_secs(0));
        }
        Some(Duration::from_secs_f64(
            elapsed * (remaining as f64 / processed as f64),
        ))
    }

    pub fn search(&self, input: SearchInput) -> Result<SearchResponse> {
        self.search_with_progress(input, None::<fn(&SearchProgress)>)
    }

    pub fn search_with_progress<F: Fn(&SearchProgress)>(
        &self,
        input: SearchInput,
        progress: Option<F>,
    ) -> Result<SearchResponse> {
        self.telemetry.reset();
        let search_request = self.build_search_request(input)?;
        let dense_model = search_request.dense_model.clone();
        let plan = self.resolve_search_plan(&search_request)?;
        let embedder = self.resolve_embedder_for_plan(&plan, &dense_model)?;

        run_search_with_plan_and_progress(
            &plan,
            &search_request,
            self.ignore.as_ref(),
            &LocalFileCorpusRepository,
            embedder,
            progress
                .as_ref()
                .map(|callback| callback as &dyn Fn(&SearchProgress)),
        )
    }

    pub fn telemetry_snapshot(&self) -> SearchTelemetry {
        SearchTelemetry::capture(self.telemetry.as_ref())
    }

    pub fn assemble_context(
        &self,
        request: ContextAssemblyRequest,
    ) -> Result<ContextAssemblyResponse> {
        self.assemble_context_with_plan(request, None, "assembly", None)
    }

    fn build_search_request(&self, input: SearchInput) -> Result<SearchRequest> {
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
        Ok(SearchRequest {
            strategy,
            query: input.query,
            intent,
            path: input.path,
            limit,
            shortlist,
            dense_model,
            rerank_model,
            gemma_model,
            retriever_timeout_ms: input.options.retriever_timeout_ms,
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
            local_context: input.options.local_context.clone(),
        })
    }

    pub fn search_turn(&self, request: SearchTurnRequest) -> Result<SearchTurnResponse> {
        let plan = self.resolve_turn_plan(&request)?;
        let assembly = self.assemble_context_with_plan(
            ContextAssemblyRequest::new(&request.path, request.query.clone())
                .with_strategy(plan.name.clone())
                .with_intent_opt(request.intent.clone())
                .with_limit(request.limit.unwrap_or(self.config.search.limit))
                .with_shortlist(request.shortlist.unwrap_or(self.config.search.shortlist))
                .with_emission_mode(request.emission_mode)
                .with_local_context(request.local_context.clone())
                .with_retained_artifacts(request.retained_artifacts.clone())
                .with_budget(ContextAssemblyBudget::new(
                    request.limit.unwrap_or(self.config.search.limit).max(1),
                )),
            Some(plan.clone()),
            &request.turn_id,
            request.session_id.clone(),
        )?;
        let response = assembly.response.clone();

        let mut decisions = vec![
            SearchControllerDecision::new(SearchControllerAction::Retrieve).with_rationale(
                format!("executed {} plan for turn {}", plan.name, request.turn_id),
            ),
        ];

        if !request.retained_artifacts.is_empty() {
            decisions.push(
                SearchControllerDecision::new(SearchControllerAction::Retain).with_rationale(
                    format!(
                        "carried {} retained evidence item(s) into this turn",
                        request.retained_artifacts.len()
                    ),
                ),
            );
        }

        decisions.push(
            SearchControllerDecision::new(SearchControllerAction::Emit).with_rationale(format!(
                "emitted {} result(s) as {:?}",
                response.hits.len(),
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
            assembly,
            decisions,
            true,
            Some("single-turn search emitted a terminal response".to_string()),
        ))
    }

    pub fn search_controller(
        &self,
        request: SearchControllerRequest,
    ) -> Result<SearchControllerResponse> {
        self.search_controller_with_progress(request, None::<fn(&SearchProgress)>)
    }

    fn search_controller_with_progress<F: Fn(&SearchProgress)>(
        &self,
        request: SearchControllerRequest,
        progress: Option<F>,
    ) -> Result<SearchControllerResponse> {
        self.telemetry.reset();
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

        let merged_local_context = Self::merge_local_context_sources(
            request
                .turns
                .iter()
                .take(turn_limit)
                .flat_map(|turn| turn.local_context.clone())
                .collect(),
        );
        let mut env_request = self.build_turn_search_request(first_turn, &request.plan);
        env_request.local_context = merged_local_context;
        let dense_model = env_request.dense_model.clone();
        let embedder = self.resolve_embedder_for_plan(&request.plan, &dense_model)?;
        let prepared = crate::search::application::prepare_search_runtime_with_progress(
            &env_request,
            self.ignore.as_ref(),
            &LocalFileCorpusRepository,
            embedder.clone(),
            progress
                .as_ref()
                .map(|callback| callback as &dyn Fn(&SearchProgress)),
        )?;
        let llm_reranker = SearchServiceBuilder::load_llm_reranker(&request.plan, &env_request)?;
        let env = SearchEnvironment::new_with_plan(
            &env_request,
            request.plan.clone(),
            &prepared.corpus,
            &prepared.index,
            embedder,
            llm_reranker,
        )?;
        let total_chunks = prepared.total_chunks;

        let mut state = request.state.clone();
        let mut turn_responses = Vec::new();
        let mut trace_turns = Vec::new();
        let mut previous_turn_id = None;
        let retrieving_started_at = Instant::now();

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
            let carried_evidence = Self::merge_retained_artifacts(
                &turn_request.retained_artifacts,
                &state.retained_artifacts,
                request.retained_artifact_limit,
            );
            turn_request.retained_artifacts = carried_evidence.retained.clone();

            if let Some(ref cb) = progress {
                let started_at = Instant::now();
                cb(&SearchProgress::Embedding {
                    phase: SearchPhase::Embedding,
                    chunks_processed: 0,
                    chunks_total: total_chunks,
                    estimated_remaining: Self::estimate_remaining(started_at, 0, total_chunks),
                });
            }
            if let Some(ref cb) = progress {
                cb(&SearchProgress::Retrieving {
                    phase: SearchPhase::Retrieving,
                    turn_index: idx,
                    turns_total: turn_limit,
                    estimated_remaining: Self::estimate_remaining(
                        retrieving_started_at,
                        idx,
                        turn_limit,
                    ),
                });
            }
            let search_request = self.build_turn_search_request(&turn_request, &request.plan);
            let response = env.search(&search_request)?;
            if let Some(ref cb) = progress {
                cb(&SearchProgress::Embedding {
                    phase: SearchPhase::Embedding,
                    chunks_processed: total_chunks,
                    chunks_total: total_chunks,
                    estimated_remaining: Some(Duration::from_secs(0)),
                });
            }
            let updated_retained = Self::derive_retained_artifacts(
                &response,
                &carried_evidence.retained,
                request.retained_artifact_limit,
            );
            let continue_more = idx + 1 < turn_limit;
            if let Some(ref cb) = progress {
                cb(&SearchProgress::Ranking {
                    phase: SearchPhase::Ranking,
                    results_processed: response.hits.len(),
                    results_total: response.hits.len(),
                    estimated_remaining: Some(Duration::from_secs(0)),
                });
            }

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
                        response.hits.len()
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

            let assembly = ContextAssemblyResponse {
                response: response.clone(),
                emission: build_search_emission(
                    &response,
                    turn_request.emission_mode,
                    &turn_request.turn_id,
                    turn_request.session_id.clone(),
                ),
                retained_artifacts: updated_retained.retained.clone(),
                pruned_artifacts: pruned,
            };

            let turn_response = self.build_turn_response(
                &turn_request,
                &request.plan,
                assembly,
                decisions,
                !continue_more,
                if continue_more {
                    None
                } else {
                    Some("controller executed all planned turns".to_string())
                },
            );

            previous_turn_id = Some(turn_response.turn.turn_id.clone());
            state.retained_artifacts = updated_retained.retained;
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

    /// Advanced autonomous seam for embedders that already own a custom
    /// planner and want Sift to lower its decisions through the shared
    /// controller runtime.
    pub fn search_autonomous_with<P: AutonomousPlanner + ?Sized>(
        &self,
        request: AutonomousSearchRequest,
        planner: &P,
    ) -> Result<AutonomousSearchResponse> {
        self.search_autonomous_with_planner_progress(request, planner, None::<fn(&SearchProgress)>)
    }

    /// Like [`search_autonomous_with`](Self::search_autonomous_with) but
    /// accepts an optional progress callback that receives [`SearchProgress`]
    /// events during each execution phase.
    pub fn search_autonomous_with_planner_progress<
        P: AutonomousPlanner + ?Sized,
        F: Fn(&SearchProgress),
    >(
        &self,
        request: AutonomousSearchRequest,
        planner: &P,
        progress: Option<F>,
    ) -> Result<AutonomousSearchResponse> {
        let request = self.normalize_autonomous_request(request);
        let plan = self.resolve_autonomous_plan(&request)?;
        let mut planner_trace = planner.plan(&request)?;

        if planner_trace.planner_strategy != request.planner_strategy {
            bail!("planner trace strategy must match the autonomous request strategy");
        }
        if planner_trace.steps.len() > request.state.step_limit {
            bail!("planner trace exceeded the configured autonomous step limit");
        }
        if planner_trace.session_id.is_none() {
            planner_trace.session_id = request.session_id.clone();
        }

        if let Some(ref cb) = progress {
            let planner_started_at = Instant::now();
            let total_decisions: usize = planner_trace
                .steps
                .iter()
                .map(|step| step.decisions.len())
                .sum();
            let mut emitted_decisions = 0usize;
            for (step_index, step) in planner_trace.steps.iter().enumerate() {
                for decision in &step.decisions {
                    let estimated_remaining = if total_decisions == 0 {
                        None
                    } else {
                        Self::estimate_remaining(
                            planner_started_at,
                            emitted_decisions,
                            total_decisions,
                        )
                    };
                    cb(&SearchProgress::PlannerStep {
                        phase: SearchPhase::Planning,
                        step_index,
                        action: format!("{:?}", decision.action),
                        query: decision.query.clone(),
                        estimated_remaining,
                    });
                    emitted_decisions += 1;
                }
            }
        }

        if request.mode == AutonomousSearchMode::Graph {
            return self.search_graph_autonomous(request, plan, planner_trace);
        }

        let turns = self.lower_autonomous_turns(&request, &planner_trace)?;
        let state = if turns.is_empty() {
            if !Self::planner_trace_completed(&planner_trace) {
                bail!("planner trace must emit a search decision or mark the episode complete");
            }
            self.derive_autonomous_state(&request, &planner_trace, None, None)
        } else {
            let mut controller_request = SearchControllerRequest::new(plan.clone(), turns)
                .with_state(
                    SearchControllerState::new(Self::count_autonomous_search_turns(&planner_trace))
                        .with_retained_artifacts(request.state.retained_artifacts.clone()),
                )
                .with_retained_artifact_limit(request.retained_artifact_limit);
            if let Some(session_id) = request.session_id.clone() {
                controller_request = controller_request.with_session_id(session_id);
            }
            let controller_response =
                self.search_controller_with_progress(controller_request, progress)?;

            let state = self.derive_autonomous_state(
                &request,
                &planner_trace,
                Some(&controller_response.state),
                None,
            );
            return Ok(AutonomousSearchResponse {
                root_task: request.root_task,
                mode: request.mode,
                planner_strategy: request.planner_strategy,
                plan,
                state,
                turns: controller_response.turns,
                planner_trace,
                trace: controller_response.trace,
            });
        };

        let completed = state.completed;
        Ok(AutonomousSearchResponse {
            root_task: request.root_task,
            mode: request.mode,
            planner_strategy: request.planner_strategy,
            plan,
            state,
            turns: Vec::new(),
            planner_trace: planner_trace.clone(),
            trace: SearchTrace {
                session_id: planner_trace.session_id.clone(),
                turns: Vec::new(),
                completed,
                termination_reason: planner_trace
                    .stop_reason
                    .map(Self::format_autonomous_stop_reason),
            },
        })
    }

    fn normalize_autonomous_request(
        &self,
        mut request: AutonomousSearchRequest,
    ) -> AutonomousSearchRequest {
        if request.mode == AutonomousSearchMode::Graph && request.state.graph_episode.is_none() {
            let seeded_graph_episode = Self::seed_graph_episode(&request);
            request.state = request.state.with_graph_episode(seeded_graph_episode);
        }
        request
    }

    fn seed_graph_episode(request: &AutonomousSearchRequest) -> AutonomousGraphEpisodeState {
        let root_node_id = request.state.current_step.step_id.clone();
        let root_branch_id = "branch-root".to_string();

        AutonomousGraphEpisodeState::new()
            .with_root_node_id(root_node_id.clone())
            .with_active_branch_id(root_branch_id.clone())
            .with_nodes(vec![
                AutonomousGraphNode::new(
                    root_node_id.clone(),
                    root_branch_id.clone(),
                    request.state.current_step.clone(),
                )
                .with_query(request.root_task.clone()),
            ])
            .with_branches(vec![
                AutonomousGraphBranchState::new(root_branch_id, root_node_id)
                    .with_status(AutonomousGraphBranchStatus::Active)
                    .with_retained_artifacts(request.state.retained_artifacts.clone()),
            ])
    }

    fn search_graph_autonomous(
        &self,
        request: AutonomousSearchRequest,
        plan: SearchPlan,
        planner_trace: AutonomousPlannerTrace,
    ) -> Result<AutonomousSearchResponse> {
        let mut graph_episode = request
            .state
            .graph_episode
            .clone()
            .unwrap_or_else(|| Self::seed_graph_episode(&request));
        replay_graph_trace(&graph_episode, &planner_trace)
            .map_err(|error| anyhow!(error.to_string()))?;
        let total_searches = Self::count_autonomous_search_turns(&planner_trace);
        let mut completed_searches = 0usize;
        let mut previous_turn_id = None;
        let mut turns = Vec::new();
        let mut trace_turns = Vec::new();

        for step in &planner_trace.steps {
            for decision in &step.decisions {
                replay_graph_decision(&mut graph_episode, &step.step, decision)
                    .map_err(|error| anyhow!(error.to_string()))?;

                if decision.action != AutonomousPlannerAction::Search {
                    continue;
                }

                completed_searches += 1;
                let turn_response = self.execute_graph_search_turn(
                    &request,
                    &plan,
                    &mut graph_episode,
                    decision,
                    GraphSearchTurnContext {
                        sequence: completed_searches,
                        continue_more: completed_searches < total_searches,
                        previous_turn_id: previous_turn_id.clone(),
                    },
                )?;
                previous_turn_id = Some(turn_response.turn.turn_id.clone());
                trace_turns.push(turn_response.trace.turns[0].clone());
                turns.push(turn_response);
            }
        }

        let state =
            self.derive_autonomous_state(&request, &planner_trace, None, Some(graph_episode));
        let completed = state.completed;

        Ok(AutonomousSearchResponse {
            root_task: request.root_task,
            mode: request.mode,
            planner_strategy: request.planner_strategy,
            plan,
            state,
            turns,
            planner_trace: planner_trace.clone(),
            trace: SearchTrace {
                session_id: planner_trace.session_id.clone(),
                turns: trace_turns,
                completed,
                termination_reason: planner_trace
                    .stop_reason
                    .map(Self::format_autonomous_stop_reason),
            },
        })
    }

    /// Supported crate-root autonomous entry point.
    ///
    /// `search_autonomous` selects the built-in planner from
    /// `request.planner_strategy` and returns both the planner trace and the
    /// lowered search/controller trace in the public response contract, so
    /// embedders do not need custom planner injection for the shipped
    /// heuristic or model-driven modes.
    pub fn search_autonomous(
        &self,
        request: AutonomousSearchRequest,
    ) -> Result<AutonomousSearchResponse> {
        self.search_autonomous_with_progress(request, None::<fn(&SearchProgress)>)
    }

    /// Like [`search_autonomous`](Self::search_autonomous) but accepts an
    /// optional progress callback that receives [`SearchProgress`] events
    /// during each execution phase.
    pub fn search_autonomous_with_progress<F: Fn(&SearchProgress)>(
        &self,
        request: AutonomousSearchRequest,
        progress: Option<F>,
    ) -> Result<AutonomousSearchResponse> {
        match request.planner_strategy.kind {
            AutonomousPlannerStrategyKind::Heuristic => {
                let planner = HeuristicAutonomousPlanner::default();
                self.search_autonomous_with_planner_progress(request, &planner, progress)
            }
            AutonomousPlannerStrategyKind::ModelDriven => {
                let planner_strategy = request.planner_strategy.clone();
                let strategy = planner_strategy
                    .profile
                    .clone()
                    .or_else(|| request.strategy.clone())
                    .unwrap_or_else(|| self.config.search.strategy.clone());
                let model = self
                    .generative(SearchOptions::default().with_strategy(strategy.clone()))
                    .map_err(|error| {
                        anyhow!(
                            "failed to resolve model-driven planner profile '{}': {error}",
                            strategy
                        )
                    })?;
                let planner = ModelDrivenAutonomousPlanner::new(model);
                self.search_autonomous_with_planner_progress(request, &planner, progress)
            }
        }
    }

    pub fn generative(&self, options: SearchOptions) -> Result<Arc<dyn GenerativeModel>> {
        if let Some(model) = &self.generative_model {
            return Ok(model.clone());
        }

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

    fn resolve_autonomous_plan(&self, request: &AutonomousSearchRequest) -> Result<SearchPlan> {
        if let Some(plan) = &request.plan {
            return Ok(plan.clone());
        }

        let strategy = request
            .strategy
            .as_deref()
            .unwrap_or(&self.config.search.strategy);
        StrategyPresetRegistry::default_registry().resolve(strategy)
    }

    fn resolve_search_plan(&self, request: &SearchRequest) -> Result<SearchPlan> {
        let mut plan = StrategyPresetRegistry::default_registry().resolve(&request.strategy)?;

        if let Some(retrievers) = &request.retrievers {
            plan.retrievers = retrievers.clone();
        }
        if let Some(fusion) = request.fusion {
            plan.fusion = fusion;
        }
        if let Some(reranking) = request.reranking {
            plan.reranking = reranking;
        }

        Ok(plan)
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
            retriever_timeout_ms: None,
            query_cache: Some(self.query_cache.clone()),
            cache_dir: self.cache_dir.clone(),
            telemetry: self.telemetry.clone(),
            prompts: Some(self.config.prompts.clone()),
            local_context: request.local_context.clone(),
        }
    }

    fn build_context_search_request(
        &self,
        request: &ContextAssemblyRequest,
        plan: &SearchPlan,
    ) -> SearchRequest {
        SearchRequest {
            query: request.query.clone(),
            intent: request.intent.clone(),
            path: request.path.clone(),
            strategy: plan.name.clone(),
            limit: request.limit.unwrap_or(self.config.search.limit),
            shortlist: request.shortlist.unwrap_or(self.config.search.shortlist),
            verbose: 0,
            retrievers: None,
            fusion: None,
            reranking: None,
            dense_model: self.default_dense_model(),
            rerank_model: self.resolve_rerank_model_for_plan(plan),
            gemma_model: self.resolve_gemma_model_for_plan(plan),
            retriever_timeout_ms: None,
            query_cache: Some(self.query_cache.clone()),
            cache_dir: self.cache_dir.clone(),
            telemetry: self.telemetry.clone(),
            prompts: Some(self.config.prompts.clone()),
            local_context: request.local_context.clone(),
        }
    }

    fn assemble_context_with_plan(
        &self,
        request: ContextAssemblyRequest,
        explicit_plan: Option<SearchPlan>,
        turn_id: &str,
        session_id: Option<String>,
    ) -> Result<ContextAssemblyResponse> {
        let plan = if let Some(plan) = explicit_plan.or_else(|| request.plan.clone()) {
            plan
        } else {
            let strategy = request
                .strategy
                .clone()
                .unwrap_or_else(|| self.config.search.strategy.clone());
            StrategyPresetRegistry::default_registry().resolve(&strategy)?
        };
        let search_request = self.build_context_search_request(&request, &plan);
        let dense_model = search_request.dense_model.clone();
        let embedder = self.resolve_embedder_for_plan(&plan, &dense_model)?;
        let response = run_search_with_plan(
            &plan,
            &search_request,
            self.ignore.as_ref(),
            &LocalFileCorpusRepository,
            embedder,
        )?;
        let retained_artifacts = Self::derive_retained_artifacts(
            &response,
            &request.retained_artifacts,
            request.budget.max_retained_artifacts,
        );

        Ok(ContextAssemblyResponse {
            response: response.clone(),
            emission: build_search_emission(&response, request.emission_mode, turn_id, session_id),
            retained_artifacts: retained_artifacts.retained,
            pruned_artifacts: retained_artifacts.pruned,
        })
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
        assembly: ContextAssemblyResponse,
        decisions: Vec<SearchControllerDecision>,
        completed: bool,
        termination_reason: Option<String>,
    ) -> SearchTurnResponse {
        let response = assembly.response.clone();
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
            result_count: response.hits.len(),
            retained_artifacts: assembly.retained_artifacts.clone(),
        };

        let trace_turn = SearchTurnTrace {
            turn_id: turn.turn_id.clone(),
            sequence: turn.sequence,
            query: turn.query.clone(),
            strategy: turn.strategy.clone(),
            emission_mode: turn.emission_mode,
            result_count: turn.result_count,
            retained_artifacts: assembly.retained_artifacts.clone(),
            decisions,
        };

        let trace = SearchTrace {
            session_id: request.session_id.clone(),
            turns: vec![trace_turn],
            completed,
            termination_reason,
        };

        SearchTurnResponse {
            turn,
            assembly,
            trace,
            emission: build_search_emission(
                &response,
                request.emission_mode,
                &request.turn_id,
                request.session_id.clone(),
            ),
        }
    }

    fn execute_graph_search_turn(
        &self,
        request: &AutonomousSearchRequest,
        plan: &SearchPlan,
        graph_episode: &mut AutonomousGraphEpisodeState,
        decision: &crate::search::AutonomousPlannerDecision,
        context: GraphSearchTurnContext,
    ) -> Result<SearchTurnResponse> {
        let GraphSearchTurnContext {
            sequence,
            continue_more,
            previous_turn_id,
        } = context;
        let branch_id = decision
            .branch_id
            .as_ref()
            .ok_or_else(|| anyhow!("graph search decisions require branch_id"))?;
        let node_id = decision
            .node_id
            .as_ref()
            .ok_or_else(|| anyhow!("graph search decisions require node_id"))?;
        let query = decision
            .query
            .clone()
            .ok_or_else(|| anyhow!("graph search decisions require a query"))?;
        let branch_index = graph_episode
            .branches
            .iter()
            .position(|branch| branch.branch_id == *branch_id)
            .ok_or_else(|| {
                anyhow!("graph search decision references unknown branch '{branch_id}'")
            })?;
        let node_index = graph_episode
            .nodes
            .iter()
            .position(|node| node.node_id == *node_id)
            .ok_or_else(|| anyhow!("graph search decision references unknown node '{node_id}'"))?;

        let mut turn_request = SearchTurnRequest::new(&request.path, query)
            .with_turn_id(
                decision
                    .turn_id
                    .clone()
                    .unwrap_or_else(|| format!("turn-{sequence}")),
            )
            .with_sequence(sequence)
            .with_strategy(plan.name.clone())
            .with_plan(plan.clone())
            .with_intent_opt(request.intent.clone())
            .with_verbose(request.verbose)
            .with_emission_mode(request.emission_mode)
            .with_retained_artifacts(
                graph_episode.branches[branch_index]
                    .retained_artifacts
                    .clone(),
            )
            .with_local_context(request.local_context.clone());

        if let Some(session_id) = &request.session_id {
            turn_request = turn_request.with_session_id(session_id.clone());
        }
        if let Some(parent_turn_id) = previous_turn_id {
            turn_request = turn_request.with_parent_turn_id(parent_turn_id);
        }
        if let Some(limit) = request.limit {
            turn_request = turn_request.with_limit(limit);
        }
        if let Some(shortlist) = request.shortlist {
            turn_request = turn_request.with_shortlist(shortlist);
        }

        let assembly = self.assemble_context_with_plan(
            ContextAssemblyRequest::new(&turn_request.path, turn_request.query.clone())
                .with_strategy(plan.name.clone())
                .with_intent_opt(turn_request.intent.clone())
                .with_limit(turn_request.limit.unwrap_or(self.config.search.limit))
                .with_shortlist(
                    turn_request
                        .shortlist
                        .unwrap_or(self.config.search.shortlist),
                )
                .with_emission_mode(turn_request.emission_mode)
                .with_local_context(turn_request.local_context.clone())
                .with_retained_artifacts(turn_request.retained_artifacts.clone())
                .with_budget(ContextAssemblyBudget::new(
                    request.retained_artifact_limit.max(1),
                )),
            Some(plan.clone()),
            &turn_request.turn_id,
            turn_request.session_id.clone(),
        )?;

        graph_episode.active_branch_id = Some(branch_id.clone());
        graph_episode.branches[branch_index].head_node_id = node_id.clone();
        graph_episode.branches[branch_index].retained_artifacts =
            assembly.retained_artifacts.clone();
        graph_episode.nodes[node_index].query = Some(turn_request.query.clone());
        graph_episode.nodes[node_index].turn_id = Some(turn_request.turn_id.clone());

        let mut decisions = vec![
            SearchControllerDecision::new(SearchControllerAction::Retrieve).with_rationale(
                format!(
                    "executed graph branch {} node {} through the shared {} plan",
                    branch_id, node_id, plan.name
                ),
            ),
        ];

        if !assembly.retained_artifacts.is_empty() {
            decisions.push(
                SearchControllerDecision::new(SearchControllerAction::Retain).with_rationale(
                    format!(
                        "retained {} branch-local evidence item(s) for {}",
                        assembly.retained_artifacts.len(),
                        branch_id
                    ),
                ),
            );
        }
        if assembly.pruned_artifacts > 0 {
            decisions.push(
                SearchControllerDecision::new(SearchControllerAction::Prune).with_rationale(
                    format!(
                        "pruned {} branch-local evidence item(s) to preserve the graph budget",
                        assembly.pruned_artifacts
                    ),
                ),
            );
        }
        decisions.push(
            SearchControllerDecision::new(SearchControllerAction::Emit).with_rationale(format!(
                "emitted {} result(s) for graph branch {}",
                assembly.response.hits.len(),
                branch_id
            )),
        );
        decisions.push(
            SearchControllerDecision::new(if continue_more {
                SearchControllerAction::Continue
            } else {
                SearchControllerAction::Terminate
            })
            .with_rationale(if continue_more {
                "continuing to the next bounded graph search turn"
            } else {
                "completed the bounded graph search turn budget"
            }),
        );

        Ok(self.build_turn_response(
            &turn_request,
            plan,
            assembly,
            decisions,
            !continue_more,
            if continue_more {
                None
            } else {
                Some("graph runtime executed all planned graph search turns".to_string())
            },
        ))
    }

    fn lower_autonomous_turns(
        &self,
        request: &AutonomousSearchRequest,
        planner_trace: &AutonomousPlannerTrace,
    ) -> Result<Vec<SearchTurnRequest>> {
        let mut turns = Vec::new();

        for step in &planner_trace.steps {
            for decision in &step.decisions {
                if decision.action != AutonomousPlannerAction::Search {
                    continue;
                }

                let query = decision
                    .query
                    .clone()
                    .ok_or_else(|| anyhow!("autonomous search decisions require a query"))?;
                let mut turn = SearchTurnRequest::new(&request.path, query)
                    .with_turn_id(
                        decision
                            .turn_id
                            .clone()
                            .unwrap_or_else(|| format!("turn-{}", turns.len() + 1)),
                    )
                    .with_intent_opt(request.intent.clone())
                    .with_verbose(request.verbose)
                    .with_emission_mode(request.emission_mode)
                    .with_local_context(request.local_context.clone());

                if let Some(session_id) = &request.session_id {
                    turn = turn.with_session_id(session_id.clone());
                }
                if let Some(limit) = request.limit {
                    turn = turn.with_limit(limit);
                }
                if let Some(shortlist) = request.shortlist {
                    turn = turn.with_shortlist(shortlist);
                }

                turns.push(turn);
            }
        }

        Ok(turns)
    }

    fn count_autonomous_search_turns(planner_trace: &AutonomousPlannerTrace) -> usize {
        planner_trace
            .steps
            .iter()
            .flat_map(|step| step.decisions.iter())
            .filter(|decision| decision.action == AutonomousPlannerAction::Search)
            .count()
    }

    fn planner_trace_completed(planner_trace: &AutonomousPlannerTrace) -> bool {
        planner_trace.completed || planner_trace.stop_reason.is_some()
    }

    fn derive_autonomous_state(
        &self,
        request: &AutonomousSearchRequest,
        planner_trace: &AutonomousPlannerTrace,
        controller_state: Option<&SearchControllerState>,
        graph_episode: Option<AutonomousGraphEpisodeState>,
    ) -> crate::search::AutonomousPlannerState {
        let current_step = planner_trace
            .steps
            .last()
            .and_then(|step| {
                step.decisions
                    .iter()
                    .rev()
                    .find_map(|decision| decision.next_step.clone())
            })
            .or_else(|| planner_trace.steps.last().map(|step| step.step.clone()))
            .unwrap_or_else(|| request.state.current_step.clone());
        let retained_artifacts = graph_episode
            .as_ref()
            .and_then(|episode| {
                episode.active_branch_id.as_ref().and_then(|branch_id| {
                    episode
                        .branches
                        .iter()
                        .find(|branch| branch.branch_id == *branch_id)
                        .map(|branch| branch.retained_artifacts.clone())
                })
            })
            .or_else(|| controller_state.map(|state| state.retained_artifacts.clone()))
            .unwrap_or_else(|| request.state.retained_artifacts.clone());
        let graph_completed = graph_episode
            .as_ref()
            .map(|episode| episode.completed)
            .unwrap_or(false);

        crate::search::AutonomousPlannerState {
            current_step,
            step_limit: request.state.step_limit,
            retained_artifacts,
            graph_episode: graph_episode.or_else(|| request.state.graph_episode.clone()),
            completed: Self::planner_trace_completed(planner_trace)
                || request.state.completed
                || graph_completed,
        }
    }

    fn format_autonomous_stop_reason(reason: AutonomousPlannerStopReason) -> String {
        match reason {
            AutonomousPlannerStopReason::GoalSatisfied => "planner terminated: goal satisfied",
            AutonomousPlannerStopReason::StepLimitReached => {
                "planner terminated: step limit reached"
            }
            AutonomousPlannerStopReason::NoFurtherQueries => {
                "planner terminated: no further queries available"
            }
            AutonomousPlannerStopReason::NoAdditionalEvidence => {
                "planner terminated: no additional evidence found"
            }
        }
        .to_string()
    }

    fn merge_retained_artifacts(
        primary: &[crate::search::RetainedArtifact],
        secondary: &[crate::search::RetainedArtifact],
        limit: usize,
    ) -> BoundedRetainedArtifacts {
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

        BoundedRetainedArtifacts {
            retained: unique.into_iter().take(limit).collect(),
            pruned,
        }
    }

    fn merge_local_context_sources(
        sources: Vec<crate::search::LocalContextSource>,
    ) -> Vec<crate::search::LocalContextSource> {
        let mut unique = Vec::new();
        let mut seen = HashSet::new();

        for source in sources {
            let key = serde_json::to_string(&source).unwrap_or_default();
            if seen.insert(key) {
                unique.push(source);
            }
        }

        unique
    }

    fn derive_retained_artifacts(
        response: &SearchResponse,
        prior: &[crate::search::RetainedArtifact],
        limit: usize,
    ) -> BoundedRetainedArtifacts {
        let fresh: Vec<_> = response
            .hits
            .iter()
            .take(limit)
            .map(|hit| {
                let mut evidence = crate::search::RetainedArtifact::new(
                    hit.artifact_id.clone(),
                    hit.artifact_kind,
                    hit.path.clone(),
                    hit.provenance.clone(),
                    hit.freshness.clone(),
                    hit.budget.clone(),
                );
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

        Self::merge_retained_artifacts(&fresh, prior, limit)
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

fn build_search_emission(
    response: &SearchResponse,
    emission_mode: SearchEmissionMode,
    turn_id: &str,
    session_id: Option<String>,
) -> SearchEmission {
    match emission_mode {
        SearchEmissionMode::View => SearchEmission::View(response.clone()),
        SearchEmissionMode::Protocol => SearchEmission::Protocol(ProtocolSearchEmission {
            turn_id: turn_id.to_string(),
            session_id,
            strategy: response.strategy.clone(),
            root: response.root.clone(),
            hits: response.hits.clone(),
        }),
        SearchEmissionMode::Latent => SearchEmission::Latent(LatentSearchEmission {
            turn_id: turn_id.to_string(),
            session_id,
            feature_space: "ranking-score".to_string(),
            hits: response
                .hits
                .iter()
                .map(|hit| LatentSearchHit {
                    artifact_id: hit.artifact_id.clone(),
                    path: hit.path.clone(),
                    score: hit.score,
                    confidence: hit.confidence,
                    location: hit.location.clone(),
                })
                .collect(),
        }),
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
    retriever_timeout_ms: Option<u64>,
    retrievers: Option<Vec<Retriever>>,
    fusion: Option<Fusion>,
    reranking: Option<Reranking>,
    cache_dir: Option<PathBuf>,
    local_context: Vec<crate::search::LocalContextSource>,
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

    pub fn with_retriever_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.retriever_timeout_ms = Some(timeout_ms);
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

    pub fn with_local_context(
        mut self,
        local_context: Vec<crate::search::LocalContextSource>,
    ) -> Self {
        self.local_context = local_context;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::{SectorMap, resolve_sector_member_path, sector_bm25_shard_cache_path};
    use std::sync::Mutex;

    use tempfile::tempdir;

    fn write_test_corpus(root: &Path) {
        std::fs::write(
            root.join("alpha.txt"),
            "alpha retrieval architecture and cache invalidation",
        )
        .expect("write alpha doc");
        std::fs::write(
            root.join("beta.txt"),
            "beta service catalog and latency measurements",
        )
        .expect("write beta doc");
    }

    fn write_sector_test_corpus(root: &Path) {
        for index in 0..6 {
            std::fs::write(
                root.join(format!("doc-{index}.txt")),
                format!(
                    "document {index} discusses retrieval sectors and cache invalidation {}",
                    "alpha ".repeat(index + 1)
                ),
            )
            .expect("write sector test doc");
        }
    }

    fn bm25_input(root: &Path) -> SearchInput {
        SearchInput::new(root, "alpha retrieval")
            .with_options(SearchOptions::default().with_strategy("bm25").with_limit(5))
    }

    fn bm25_controller_request(root: &Path) -> SearchControllerRequest {
        let plan = StrategyPresetRegistry::default_registry()
            .resolve("bm25")
            .expect("bm25 plan");
        SearchControllerRequest::new(plan, vec![SearchTurnRequest::new(root, "alpha retrieval")])
    }

    fn bm25_autonomous_request(root: &Path) -> AutonomousSearchRequest {
        AutonomousSearchRequest::new(root, "alpha retrieval").with_strategy("bm25")
    }

    fn assert_sector_warm_reuse(telemetry: &SearchTelemetry) {
        assert!(telemetry.sector_cache_hits > 0);
        assert_eq!(telemetry.fresh_artifact_builds, 0);
        assert!(telemetry.bm25_index_cache_hits > 0);
    }

    #[test]
    fn direct_search_with_progress_emits_indexing_and_ranking() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_test_corpus(corpus.path());

        let engine = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = events.clone();

        let response = engine
            .search_with_progress(
                bm25_input(corpus.path()),
                Some(move |event: &SearchProgress| {
                    captured.lock().expect("progress lock").push(event.clone());
                }),
            )
            .expect("direct search with progress");

        assert!(!response.hits.is_empty());

        let events = events.lock().expect("events lock");
        assert!(
            events
                .iter()
                .any(|event| matches!(event, SearchProgress::Indexing { .. }))
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, SearchProgress::Ranking { .. }))
        );
        assert!(events.iter().any(|event| matches!(
            event,
            SearchProgress::Indexing {
                files_processed,
                files_total,
                ..
            } if files_processed == files_total
        )));
        assert_eq!(
            response.coverage.mode,
            crate::search::SearchCoverageMode::Sealed
        );
    }

    #[test]
    fn direct_search_progress_surfaces_frontier_then_converging_then_sealed() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_sector_test_corpus(corpus.path());

        let first = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        first
            .search(bm25_input(corpus.path()))
            .expect("first direct search");

        let before =
            SectorMap::load_for_root(cache.path(), corpus.path()).expect("before sector map");
        assert!(before.sectors.len() >= 2);
        let changed_sector = before.sectors.first().expect("sector");
        let changed_path = resolve_sector_member_path(
            corpus.path(),
            &changed_sector.proofs.first().expect("proof").relative_path,
        );
        std::fs::write(
            &changed_path,
            "coverage transition test forces one dirty sector rebuild",
        )
        .expect("rewrite changed path");

        let second = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&events);

        let response = second
            .search_with_progress(
                bm25_input(corpus.path()),
                Some(move |event: &SearchProgress| {
                    captured.lock().expect("progress lock").push(event.clone());
                }),
            )
            .expect("search with coverage progress");

        assert_eq!(
            response.coverage.mode,
            crate::search::SearchCoverageMode::Sealed
        );
        assert_eq!(response.coverage.dirty_sector_count, 0);
        assert!(response.coverage.completed_dirty_sector_count > 0);

        let indexing_modes = events
            .lock()
            .expect("events lock")
            .iter()
            .filter_map(|event| match event {
                SearchProgress::Indexing { coverage, .. } => Some(coverage.mode),
                _ => None,
            })
            .collect::<Vec<_>>();

        let frontier_index = indexing_modes
            .iter()
            .position(|mode| *mode == crate::search::SearchCoverageMode::Frontier)
            .expect("frontier mode should be reported");
        let converging_index = indexing_modes
            .iter()
            .position(|mode| *mode == crate::search::SearchCoverageMode::Converging)
            .expect("converging mode should be reported");
        let sealed_index = indexing_modes
            .iter()
            .rposition(|mode| *mode == crate::search::SearchCoverageMode::Sealed)
            .expect("sealed mode should be reported");

        assert!(frontier_index < converging_index);
        assert!(converging_index < sealed_index);
    }

    #[test]
    fn direct_search_reuses_bm25_cache() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_test_corpus(corpus.path());

        let first = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        first
            .search(bm25_input(corpus.path()))
            .expect("first direct search");
        let first_telemetry = first.telemetry_snapshot();
        assert!(first_telemetry.bm25_index_builds > 0 || first_telemetry.sector_shard_builds > 0);

        let second = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        second
            .search(bm25_input(corpus.path()))
            .expect("second direct search");
        assert!(second.telemetry_snapshot().bm25_index_cache_hits > 0);
    }

    #[test]
    fn direct_search_reuses_clean_sectors_on_warm_restart() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_sector_test_corpus(corpus.path());

        let first = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        first
            .search(bm25_input(corpus.path()))
            .expect("first direct search");

        let sector_map = SectorMap::load_for_root(cache.path(), corpus.path()).expect("sector map");
        assert!(!sector_map.sectors.is_empty());
        for sector in &sector_map.sectors {
            let shard = sector
                .shards
                .bm25
                .as_ref()
                .expect("sector shard should be persisted");
            assert!(
                sector_bm25_shard_cache_path(
                    cache.path(),
                    corpus.path(),
                    &sector.sector_id,
                    &shard.key
                )
                .exists()
            );
        }

        let second = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        second
            .search(bm25_input(corpus.path()))
            .expect("second direct search");

        let telemetry = second.telemetry_snapshot();
        assert!(telemetry.sector_cache_hits > 0);
        assert_eq!(telemetry.fresh_artifact_builds, 0);
        assert!(telemetry.bm25_index_cache_hits > 0);
    }

    #[test]
    fn direct_search_rebuilds_only_the_dirty_sector() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_sector_test_corpus(corpus.path());

        let first = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        first
            .search(bm25_input(corpus.path()))
            .expect("first direct search");

        let before =
            SectorMap::load_for_root(cache.path(), corpus.path()).expect("before sector map");
        assert!(before.sectors.len() >= 2);
        let changed_sector = before.sectors.first().expect("sector");
        let changed_path = resolve_sector_member_path(
            corpus.path(),
            &changed_sector.proofs.first().expect("proof").relative_path,
        );
        std::fs::write(
            &changed_path,
            "changed retrieval sector content with a targeted dirty rebuild",
        )
        .expect("rewrite changed path");

        let second = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        second
            .search(bm25_input(corpus.path()))
            .expect("second direct search");

        let after =
            SectorMap::load_for_root(cache.path(), corpus.path()).expect("after sector map");
        let before_by_id = before
            .sectors
            .iter()
            .map(|sector| (sector.sector_id.clone(), sector))
            .collect::<std::collections::HashMap<_, _>>();
        let changed = after
            .sectors
            .iter()
            .filter(|sector| {
                before_by_id
                    .get(&sector.sector_id)
                    .map(|previous| {
                        previous.membership.proof_fingerprint != sector.membership.proof_fingerprint
                            || previous
                                .shards
                                .bm25
                                .as_ref()
                                .map(|shard| shard.key.as_str())
                                != sector.shards.bm25.as_ref().map(|shard| shard.key.as_str())
                    })
                    .unwrap_or(true)
            })
            .map(|sector| sector.sector_id.clone())
            .collect::<Vec<_>>();

        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0], changed_sector.sector_id);

        let telemetry = second.telemetry_snapshot();
        assert_eq!(telemetry.sector_rebuilds, 1);
        assert!(telemetry.sector_cache_hits >= before.sectors.len().saturating_sub(1));
        assert!(telemetry.sector_shard_cache_hits > 0);
    }

    #[test]
    fn search_controller_reuses_bm25_cache() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_test_corpus(corpus.path());

        let request = bm25_controller_request(corpus.path());

        let first = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        first
            .search_controller(request.clone())
            .expect("first controller search");
        let first_telemetry = first.telemetry_snapshot();
        assert!(first_telemetry.bm25_index_builds > 0 || first_telemetry.sector_shard_builds > 0);

        let second = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        second
            .search_controller(request)
            .expect("second controller search");
        assert!(second.telemetry_snapshot().bm25_index_cache_hits > 0);
    }

    #[test]
    fn search_controller_reuses_clean_sectors_on_warm_restart() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_sector_test_corpus(corpus.path());
        let request = bm25_controller_request(corpus.path());

        let first = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        first
            .search_controller(request.clone())
            .expect("first controller search");

        let second = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        second
            .search_controller(request)
            .expect("second controller search");

        let telemetry = second.telemetry_snapshot();
        assert!(telemetry.sector_cache_hits > 0);
        assert_eq!(telemetry.fresh_artifact_builds, 0);
        assert!(telemetry.bm25_index_cache_hits > 0);
    }

    #[test]
    fn autonomous_search_reuses_clean_sectors_on_warm_restart() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_sector_test_corpus(corpus.path());
        let request = bm25_autonomous_request(corpus.path());

        let first = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        first
            .search_autonomous(request.clone())
            .expect("first autonomous search");

        let second = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        second
            .search_autonomous(request)
            .expect("second autonomous search");

        assert_sector_warm_reuse(&second.telemetry_snapshot());
    }

    #[test]
    fn controller_reuses_sectors_prepared_by_direct_search() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_sector_test_corpus(corpus.path());

        let direct = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        direct
            .search(bm25_input(corpus.path()))
            .expect("direct search primes cache");

        let controller = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        controller
            .search_controller(bm25_controller_request(corpus.path()))
            .expect("controller search reuses cache");

        assert_sector_warm_reuse(&controller.telemetry_snapshot());
    }

    #[test]
    fn direct_search_reuses_sectors_prepared_by_controller() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_sector_test_corpus(corpus.path());

        let controller = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        controller
            .search_controller(bm25_controller_request(corpus.path()))
            .expect("controller search primes cache");

        let direct = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        direct
            .search(bm25_input(corpus.path()))
            .expect("direct search reuses cache");

        assert_sector_warm_reuse(&direct.telemetry_snapshot());
    }

    #[test]
    fn direct_search_reuses_sectors_prepared_by_autonomous_search() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_sector_test_corpus(corpus.path());

        let autonomous = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        autonomous
            .search_autonomous(bm25_autonomous_request(corpus.path()))
            .expect("autonomous search primes cache");

        let direct = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        direct
            .search(bm25_input(corpus.path()))
            .expect("direct search reuses autonomous cache");

        assert_sector_warm_reuse(&direct.telemetry_snapshot());
    }

    #[test]
    fn cross_surface_dirty_rebuilds_stay_bounded() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_sector_test_corpus(corpus.path());

        let controller = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        controller
            .search_controller(bm25_controller_request(corpus.path()))
            .expect("controller search primes cache");

        let before =
            SectorMap::load_for_root(cache.path(), corpus.path()).expect("before sector map");
        assert!(before.sectors.len() >= 2);
        let changed_sector = before.sectors.first().expect("sector");
        let changed_path = resolve_sector_member_path(
            corpus.path(),
            &changed_sector.proofs.first().expect("proof").relative_path,
        );
        std::fs::write(
            &changed_path,
            "cross-surface dirty rebuild with shared cache reuse",
        )
        .expect("rewrite changed path");

        let direct = Sift::builder()
            .without_ignore()
            .with_cache_dir(cache.path())
            .build();
        direct
            .search(bm25_input(corpus.path()))
            .expect("direct search reuses controller cache");

        let telemetry = direct.telemetry_snapshot();
        assert_eq!(telemetry.sector_rebuilds, 1);
        assert!(telemetry.sector_cache_hits >= before.sectors.len().saturating_sub(1));
        assert!(telemetry.sector_shard_cache_hits > 0);
    }
}
