use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub use crate::internal::config::Ignore;
use anyhow::{Result, anyhow};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::dense::DenseModelSpec;
use crate::extract::SourceKind;
use crate::segment::Segment;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    Bm25,
    Hybrid,
}

impl Engine {
    pub fn is_hybrid(&self) -> bool {
        matches!(self, Engine::Hybrid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchPlan {
    pub name: String,
    pub query_expansion: QueryExpansionPolicy,
    pub retrievers: Vec<RetrieverPolicy>,
    pub fusion: FusionPolicy,
    pub reranking: RerankingPolicy,
}

impl SearchPlan {
    pub fn default_lexical() -> Self {
        Self {
            name: "lexical".to_string(),
            query_expansion: QueryExpansionPolicy::None,
            retrievers: vec![RetrieverPolicy::Bm25],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::None,
        }
    }

    pub fn categorize_score(&self, score: f64) -> ScoreConfidence {
        if score > 0.8 {
            ScoreConfidence::High
        } else if score > 0.4 {
            ScoreConfidence::Medium
        } else {
            ScoreConfidence::Low
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContextArtifactKind {
    File,
    ProjectDocument,
    EnvironmentFact,
    ToolOutput,
    AgentTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcquisitionAdapterKind {
    FileSystem,
    ProjectDocument,
    EnvironmentContext,
    ToolOutput,
    AgentTurn,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactProvenance {
    pub adapter: AcquisitionAdapterKind,
    pub source: String,
    pub synthetic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactFreshness {
    pub observed_unix_secs: i64,
    pub modified_unix_secs: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactBudget {
    pub bytes: usize,
    pub token_estimate: usize,
    pub segment_count: usize,
}

impl ArtifactBudget {
    pub fn from_text(text: &str, segment_count: usize) -> Self {
        Self {
            bytes: text.len(),
            token_estimate: tokenize(text).len(),
            segment_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvironmentFactInput {
    pub key: String,
    pub value: String,
}

impl EnvironmentFactInput {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolOutputInput {
    pub tool_name: String,
    pub call_id: String,
    pub content: String,
}

impl ToolOutputInput {
    pub fn new(
        tool_name: impl Into<String>,
        call_id: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            call_id: call_id.into(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentTurnInput {
    pub session_id: Option<String>,
    pub turn_id: String,
    pub role: String,
    pub content: String,
}

impl AgentTurnInput {
    pub fn new(
        turn_id: impl Into<String>,
        role: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            session_id: None,
            turn_id: turn_id.into(),
            role: role.into(),
            content: content.into(),
        }
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "payload", rename_all = "kebab-case")]
pub enum LocalContextSource {
    EnvironmentFact(EnvironmentFactInput),
    ToolOutput(ToolOutputInput),
    AgentTurn(AgentTurnInput),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum QueryExpansionPolicy {
    None,
    Synonym,
    Hyde,
    Splade,
    Classified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum RetrieverPolicy {
    Bm25,
    Phrase,
    Vector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum FusionPolicy {
    Rrf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum RerankingPolicy {
    None,
    PositionAware,
    Llm,
    Jina,
    Gemma,
}

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub intent: Option<String>,
    pub path: PathBuf,
    pub strategy: String,
    pub limit: usize,
    pub shortlist: usize,
    pub verbose: u8,
    pub retrievers: Option<Vec<RetrieverPolicy>>,
    pub fusion: Option<FusionPolicy>,
    pub reranking: Option<RerankingPolicy>,
    pub dense_model: DenseModelSpec,
    pub rerank_model: Option<crate::search::adapters::qwen::QwenModelSpec>,
    pub gemma_model: Option<crate::search::adapters::gemma::GemmaModelSpec>,
    pub retriever_timeout_ms: Option<u64>,
    pub query_cache: Option<QueryEmbeddingCache>,
    pub cache_dir: Option<PathBuf>,
    pub telemetry: std::sync::Arc<crate::system::Telemetry>,
    pub prompts: Option<crate::config::PromptsConfig>,
    pub local_context: Vec<LocalContextSource>,
}

impl SearchRequest {
    pub fn new(strategy: &str, query: impl Into<String>, path: PathBuf) -> Self {
        Self {
            query: query.into(),
            intent: None,
            path,
            strategy: strategy.to_string(),
            limit: 10,
            shortlist: 50,
            verbose: 0,
            retrievers: None,
            fusion: None,
            reranking: None,
            dense_model: DenseModelSpec::default(),
            rerank_model: None,
            gemma_model: None,
            retriever_timeout_ms: None,
            query_cache: None,
            cache_dir: None,
            telemetry: std::sync::Arc::new(crate::system::Telemetry::new()),
            prompts: None,
            local_context: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResponse {
    pub strategy: String,
    pub root: String,
    pub indexed_artifacts: usize,
    pub skipped_artifacts: usize,
    pub hits: Vec<SearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchHit {
    pub artifact_id: String,
    pub artifact_kind: ContextArtifactKind,
    pub path: String,
    pub rank: usize,
    pub score: f64,
    pub confidence: ScoreConfidence,
    pub location: Option<String>,
    pub snippet: String,
    pub provenance: ArtifactProvenance,
    pub freshness: ArtifactFreshness,
    pub budget: ArtifactBudget,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchEmissionMode {
    #[default]
    View,
    Protocol,
    Latent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetainedArtifact {
    pub artifact_id: String,
    pub artifact_kind: ContextArtifactKind,
    pub path: String,
    pub location: Option<String>,
    pub snippet: Option<String>,
    pub rationale: Option<String>,
    pub provenance: ArtifactProvenance,
    pub freshness: ArtifactFreshness,
    pub budget: ArtifactBudget,
}

impl RetainedArtifact {
    pub fn new(
        artifact_id: impl Into<String>,
        artifact_kind: ContextArtifactKind,
        path: impl Into<String>,
        provenance: ArtifactProvenance,
        freshness: ArtifactFreshness,
        budget: ArtifactBudget,
    ) -> Self {
        Self {
            artifact_id: artifact_id.into(),
            artifact_kind,
            path: path.into(),
            location: None,
            snippet: None,
            rationale: None,
            provenance,
            freshness,
            budget,
        }
    }

    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = Some(rationale.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchTurnRequest {
    pub session_id: Option<String>,
    pub turn_id: String,
    pub parent_turn_id: Option<String>,
    pub sequence: usize,
    pub path: PathBuf,
    pub query: String,
    pub intent: Option<String>,
    pub strategy: Option<String>,
    pub plan: Option<SearchPlan>,
    pub limit: Option<usize>,
    pub shortlist: Option<usize>,
    pub verbose: u8,
    pub emission_mode: SearchEmissionMode,
    pub retained_artifacts: Vec<RetainedArtifact>,
    pub local_context: Vec<LocalContextSource>,
}

impl SearchTurnRequest {
    pub fn new(path: impl AsRef<Path>, query: impl Into<String>) -> Self {
        Self {
            session_id: None,
            turn_id: "turn-1".to_string(),
            parent_turn_id: None,
            sequence: 1,
            path: path.as_ref().to_path_buf(),
            query: query.into(),
            intent: None,
            strategy: None,
            plan: None,
            limit: None,
            shortlist: None,
            verbose: 0,
            emission_mode: SearchEmissionMode::View,
            retained_artifacts: Vec::new(),
            local_context: Vec::new(),
        }
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_turn_id(mut self, turn_id: impl Into<String>) -> Self {
        self.turn_id = turn_id.into();
        self
    }

    pub fn with_parent_turn_id(mut self, parent_turn_id: impl Into<String>) -> Self {
        self.parent_turn_id = Some(parent_turn_id.into());
        self
    }

    pub fn with_sequence(mut self, sequence: usize) -> Self {
        self.sequence = sequence;
        self
    }

    pub fn with_intent(mut self, intent: impl Into<String>) -> Self {
        self.intent = Some(intent.into());
        self
    }

    pub fn with_intent_opt(mut self, intent: Option<String>) -> Self {
        self.intent = intent;
        self
    }

    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.strategy = Some(strategy.into());
        self
    }

    pub fn with_plan(mut self, plan: SearchPlan) -> Self {
        self.plan = Some(plan);
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

    pub fn with_verbose(mut self, verbose: u8) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_emission_mode(mut self, emission_mode: SearchEmissionMode) -> Self {
        self.emission_mode = emission_mode;
        self
    }

    pub fn with_retained_artifacts(mut self, retained_artifacts: Vec<RetainedArtifact>) -> Self {
        self.retained_artifacts = retained_artifacts;
        self
    }

    pub fn with_local_context(mut self, local_context: Vec<LocalContextSource>) -> Self {
        self.local_context = local_context;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum AutonomousPlannerStrategyKind {
    Heuristic,
    ModelDriven,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousPlannerStrategy {
    pub kind: AutonomousPlannerStrategyKind,
    pub profile: Option<String>,
}

impl AutonomousPlannerStrategy {
    pub fn heuristic() -> Self {
        Self {
            kind: AutonomousPlannerStrategyKind::Heuristic,
            profile: None,
        }
    }

    pub fn model_driven() -> Self {
        Self {
            kind: AutonomousPlannerStrategyKind::ModelDriven,
            profile: None,
        }
    }

    pub fn with_profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = Some(profile.into());
        self
    }
}

impl Default for AutonomousPlannerStrategy {
    fn default() -> Self {
        Self::heuristic()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AutonomousSearchMode {
    #[default]
    Linear,
    Graph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AutonomousGraphBranchStatus {
    Pending,
    Active,
    Completed,
    Merged,
    Pruned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AutonomousGraphEdgeKind {
    Root,
    Child,
    Sibling,
    Merge,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousGraphNode {
    pub node_id: String,
    pub branch_id: String,
    pub step: AutonomousPlannerStepCursor,
    pub query: Option<String>,
    pub turn_id: Option<String>,
}

impl AutonomousGraphNode {
    pub fn new(
        node_id: impl Into<String>,
        branch_id: impl Into<String>,
        step: AutonomousPlannerStepCursor,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            branch_id: branch_id.into(),
            step,
            query: None,
            turn_id: None,
        }
    }

    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn with_turn_id(mut self, turn_id: impl Into<String>) -> Self {
        self.turn_id = Some(turn_id.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousGraphEdge {
    pub edge_id: String,
    pub from_node_id: String,
    pub to_node_id: String,
    pub kind: AutonomousGraphEdgeKind,
}

impl AutonomousGraphEdge {
    pub fn new(
        edge_id: impl Into<String>,
        from_node_id: impl Into<String>,
        to_node_id: impl Into<String>,
        kind: AutonomousGraphEdgeKind,
    ) -> Self {
        Self {
            edge_id: edge_id.into(),
            from_node_id: from_node_id.into(),
            to_node_id: to_node_id.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousGraphFrontierEntry {
    pub frontier_id: String,
    pub branch_id: String,
    pub node_id: String,
    pub priority: usize,
}

impl AutonomousGraphFrontierEntry {
    pub fn new(
        frontier_id: impl Into<String>,
        branch_id: impl Into<String>,
        node_id: impl Into<String>,
    ) -> Self {
        Self {
            frontier_id: frontier_id.into(),
            branch_id: branch_id.into(),
            node_id: node_id.into(),
            priority: 0,
        }
    }

    pub fn with_priority(mut self, priority: usize) -> Self {
        self.priority = priority;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousGraphBranchState {
    pub branch_id: String,
    pub status: AutonomousGraphBranchStatus,
    pub head_node_id: String,
    pub retained_artifacts: Vec<RetainedArtifact>,
}

impl AutonomousGraphBranchState {
    pub fn new(branch_id: impl Into<String>, head_node_id: impl Into<String>) -> Self {
        Self {
            branch_id: branch_id.into(),
            status: AutonomousGraphBranchStatus::Pending,
            head_node_id: head_node_id.into(),
            retained_artifacts: Vec::new(),
        }
    }

    pub fn with_status(mut self, status: AutonomousGraphBranchStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_head_node_id(mut self, head_node_id: impl Into<String>) -> Self {
        self.head_node_id = head_node_id.into();
        self
    }

    pub fn with_retained_artifacts(mut self, retained_artifacts: Vec<RetainedArtifact>) -> Self {
        self.retained_artifacts = retained_artifacts;
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousGraphEpisodeState {
    pub root_node_id: Option<String>,
    pub active_branch_id: Option<String>,
    pub frontier: Vec<AutonomousGraphFrontierEntry>,
    pub branches: Vec<AutonomousGraphBranchState>,
    pub nodes: Vec<AutonomousGraphNode>,
    pub edges: Vec<AutonomousGraphEdge>,
    pub completed: bool,
}

impl AutonomousGraphEpisodeState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_root_node_id(mut self, root_node_id: impl Into<String>) -> Self {
        self.root_node_id = Some(root_node_id.into());
        self
    }

    pub fn with_active_branch_id(mut self, active_branch_id: impl Into<String>) -> Self {
        self.active_branch_id = Some(active_branch_id.into());
        self
    }

    pub fn with_frontier(mut self, frontier: Vec<AutonomousGraphFrontierEntry>) -> Self {
        self.frontier = frontier;
        self
    }

    pub fn with_branches(mut self, branches: Vec<AutonomousGraphBranchState>) -> Self {
        self.branches = branches;
        self
    }

    pub fn with_nodes(mut self, nodes: Vec<AutonomousGraphNode>) -> Self {
        self.nodes = nodes;
        self
    }

    pub fn with_edges(mut self, edges: Vec<AutonomousGraphEdge>) -> Self {
        self.edges = edges;
        self
    }

    pub fn with_completed(mut self, completed: bool) -> Self {
        self.completed = completed;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousPlannerStepCursor {
    pub step_id: String,
    pub parent_step_id: Option<String>,
    pub sequence: usize,
}

impl AutonomousPlannerStepCursor {
    pub fn new(step_id: impl Into<String>, sequence: usize) -> Self {
        Self {
            step_id: step_id.into(),
            parent_step_id: None,
            sequence,
        }
    }

    pub fn first() -> Self {
        Self::new("step-1", 1)
    }

    pub fn with_parent_step_id(mut self, parent_step_id: impl Into<String>) -> Self {
        self.parent_step_id = Some(parent_step_id.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousPlannerState {
    pub current_step: AutonomousPlannerStepCursor,
    pub step_limit: usize,
    pub retained_artifacts: Vec<RetainedArtifact>,
    pub graph_episode: Option<AutonomousGraphEpisodeState>,
    pub completed: bool,
}

impl AutonomousPlannerState {
    pub fn new(step_limit: usize) -> Self {
        Self {
            current_step: AutonomousPlannerStepCursor::first(),
            step_limit,
            retained_artifacts: Vec::new(),
            graph_episode: None,
            completed: false,
        }
    }

    pub fn with_current_step(mut self, current_step: AutonomousPlannerStepCursor) -> Self {
        self.current_step = current_step;
        self
    }

    pub fn with_step_limit(mut self, step_limit: usize) -> Self {
        self.step_limit = step_limit;
        self
    }

    pub fn with_retained_artifacts(mut self, retained_artifacts: Vec<RetainedArtifact>) -> Self {
        self.retained_artifacts = retained_artifacts;
        self
    }

    pub fn with_graph_episode(mut self, graph_episode: AutonomousGraphEpisodeState) -> Self {
        self.graph_episode = Some(graph_episode);
        self
    }

    pub fn with_completed(mut self, completed: bool) -> Self {
        self.completed = completed;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousSearchRequest {
    pub session_id: Option<String>,
    pub path: PathBuf,
    pub root_task: String,
    pub mode: AutonomousSearchMode,
    pub intent: Option<String>,
    pub strategy: Option<String>,
    pub plan: Option<SearchPlan>,
    pub planner_strategy: AutonomousPlannerStrategy,
    pub state: AutonomousPlannerState,
    pub limit: Option<usize>,
    pub shortlist: Option<usize>,
    pub verbose: u8,
    pub emission_mode: SearchEmissionMode,
    pub local_context: Vec<LocalContextSource>,
    pub retained_artifact_limit: usize,
}

impl AutonomousSearchRequest {
    pub fn new(path: impl AsRef<Path>, root_task: impl Into<String>) -> Self {
        Self {
            session_id: None,
            path: path.as_ref().to_path_buf(),
            root_task: root_task.into(),
            mode: AutonomousSearchMode::Linear,
            intent: None,
            strategy: None,
            plan: None,
            planner_strategy: AutonomousPlannerStrategy::default(),
            state: AutonomousPlannerState::new(3),
            limit: None,
            shortlist: None,
            verbose: 0,
            emission_mode: SearchEmissionMode::View,
            local_context: Vec::new(),
            retained_artifact_limit: 5,
        }
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_intent(mut self, intent: impl Into<String>) -> Self {
        self.intent = Some(intent.into());
        self
    }

    pub fn with_intent_opt(mut self, intent: Option<String>) -> Self {
        self.intent = intent;
        self
    }

    pub fn with_mode(mut self, mode: AutonomousSearchMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.strategy = Some(strategy.into());
        self
    }

    pub fn with_plan(mut self, plan: SearchPlan) -> Self {
        self.plan = Some(plan);
        self
    }

    pub fn with_planner_strategy(mut self, planner_strategy: AutonomousPlannerStrategy) -> Self {
        self.planner_strategy = planner_strategy;
        self
    }

    pub fn with_state(mut self, state: AutonomousPlannerState) -> Self {
        self.state = state;
        self
    }

    pub fn with_graph_episode(mut self, graph_episode: AutonomousGraphEpisodeState) -> Self {
        self.state = self.state.with_graph_episode(graph_episode);
        self
    }

    pub fn with_step_limit(mut self, step_limit: usize) -> Self {
        self.state = self.state.with_step_limit(step_limit);
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

    pub fn with_verbose(mut self, verbose: u8) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_emission_mode(mut self, emission_mode: SearchEmissionMode) -> Self {
        self.emission_mode = emission_mode;
        self
    }

    pub fn with_local_context(mut self, local_context: Vec<LocalContextSource>) -> Self {
        self.local_context = local_context;
        self
    }

    pub fn with_retained_artifact_limit(mut self, retained_artifact_limit: usize) -> Self {
        self.retained_artifact_limit = retained_artifact_limit;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AutonomousPlannerAction {
    Search,
    Fork,
    Select,
    Merge,
    Prune,
    Continue,
    Terminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AutonomousPlannerStopReason {
    GoalSatisfied,
    StepLimitReached,
    NoFurtherQueries,
    NoAdditionalEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousPlannerDecision {
    pub action: AutonomousPlannerAction,
    pub rationale: Option<String>,
    pub query: Option<String>,
    pub turn_id: Option<String>,
    pub branch_id: Option<String>,
    pub node_id: Option<String>,
    pub target_branch_id: Option<String>,
    pub target_node_id: Option<String>,
    pub edge_id: Option<String>,
    pub edge_kind: Option<AutonomousGraphEdgeKind>,
    pub frontier_id: Option<String>,
    pub next_step: Option<AutonomousPlannerStepCursor>,
    pub stop_reason: Option<AutonomousPlannerStopReason>,
}

impl AutonomousPlannerDecision {
    pub fn new(action: AutonomousPlannerAction) -> Self {
        Self {
            action,
            rationale: None,
            query: None,
            turn_id: None,
            branch_id: None,
            node_id: None,
            target_branch_id: None,
            target_node_id: None,
            edge_id: None,
            edge_kind: None,
            frontier_id: None,
            next_step: None,
            stop_reason: None,
        }
    }

    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = Some(rationale.into());
        self
    }

    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn with_turn_id(mut self, turn_id: impl Into<String>) -> Self {
        self.turn_id = Some(turn_id.into());
        self
    }

    pub fn with_branch_id(mut self, branch_id: impl Into<String>) -> Self {
        self.branch_id = Some(branch_id.into());
        self
    }

    pub fn with_node_id(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }

    pub fn with_target_branch_id(mut self, target_branch_id: impl Into<String>) -> Self {
        self.target_branch_id = Some(target_branch_id.into());
        self
    }

    pub fn with_target_node_id(mut self, target_node_id: impl Into<String>) -> Self {
        self.target_node_id = Some(target_node_id.into());
        self
    }

    pub fn with_edge_id(mut self, edge_id: impl Into<String>) -> Self {
        self.edge_id = Some(edge_id.into());
        self
    }

    pub fn with_edge_kind(mut self, edge_kind: AutonomousGraphEdgeKind) -> Self {
        self.edge_kind = Some(edge_kind);
        self
    }

    pub fn with_frontier_id(mut self, frontier_id: impl Into<String>) -> Self {
        self.frontier_id = Some(frontier_id.into());
        self
    }

    pub fn with_next_step(mut self, next_step: AutonomousPlannerStepCursor) -> Self {
        self.next_step = Some(next_step);
        self
    }

    pub fn with_stop_reason(mut self, stop_reason: AutonomousPlannerStopReason) -> Self {
        self.stop_reason = Some(stop_reason);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousPlannerTraceStep {
    pub step: AutonomousPlannerStepCursor,
    pub decisions: Vec<AutonomousPlannerDecision>,
}

impl AutonomousPlannerTraceStep {
    pub fn new(step: AutonomousPlannerStepCursor) -> Self {
        Self {
            step,
            decisions: Vec::new(),
        }
    }

    pub fn with_decisions(mut self, decisions: Vec<AutonomousPlannerDecision>) -> Self {
        self.decisions = decisions;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousPlannerTrace {
    pub session_id: Option<String>,
    pub planner_strategy: AutonomousPlannerStrategy,
    pub steps: Vec<AutonomousPlannerTraceStep>,
    pub completed: bool,
    pub stop_reason: Option<AutonomousPlannerStopReason>,
}

impl AutonomousPlannerTrace {
    pub fn new(planner_strategy: AutonomousPlannerStrategy) -> Self {
        Self {
            session_id: None,
            planner_strategy,
            steps: Vec::new(),
            completed: false,
            stop_reason: None,
        }
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_steps(mut self, steps: Vec<AutonomousPlannerTraceStep>) -> Self {
        self.steps = steps;
        self
    }

    pub fn with_completed(mut self, completed: bool) -> Self {
        self.completed = completed;
        self
    }

    pub fn with_stop_reason(mut self, stop_reason: AutonomousPlannerStopReason) -> Self {
        self.stop_reason = Some(stop_reason);
        self
    }
}

pub trait AutonomousPlanner: Send + Sync {
    fn plan(&self, request: &AutonomousSearchRequest) -> Result<AutonomousPlannerTrace>;
}

/// Phase discriminant for downstream display mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchPhase {
    Indexing,
    Embedding,
    Planning,
    Retrieving,
    Ranking,
}

impl std::fmt::Display for SearchPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Indexing => write!(f, "Indexing"),
            Self::Embedding => write!(f, "Embedding"),
            Self::Planning => write!(f, "Planning"),
            Self::Retrieving => write!(f, "Retrieving"),
            Self::Ranking => write!(f, "Ranking"),
        }
    }
}

/// Structured progress event emitted during `search_autonomous` execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SearchProgress {
    Indexing {
        phase: SearchPhase,
        files_processed: usize,
        files_total: usize,
        estimated_remaining: Option<std::time::Duration>,
    },
    Embedding {
        phase: SearchPhase,
        chunks_processed: usize,
        chunks_total: usize,
        estimated_remaining: Option<std::time::Duration>,
    },
    PlannerStep {
        phase: SearchPhase,
        step_index: usize,
        action: String,
        query: Option<String>,
        estimated_remaining: Option<std::time::Duration>,
    },
    Retrieving {
        phase: SearchPhase,
        turn_index: usize,
        turns_total: usize,
        estimated_remaining: Option<std::time::Duration>,
    },
    Ranking {
        phase: SearchPhase,
        results_processed: usize,
        results_total: usize,
        estimated_remaining: Option<std::time::Duration>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchControllerState {
    pub next_turn: usize,
    pub turn_limit: usize,
    pub retained_artifacts: Vec<RetainedArtifact>,
    pub completed: bool,
}

impl SearchControllerState {
    pub fn new(turn_limit: usize) -> Self {
        Self {
            next_turn: 0,
            turn_limit,
            retained_artifacts: Vec::new(),
            completed: false,
        }
    }

    pub fn with_next_turn(mut self, next_turn: usize) -> Self {
        self.next_turn = next_turn;
        self
    }

    pub fn with_turn_limit(mut self, turn_limit: usize) -> Self {
        self.turn_limit = turn_limit;
        self
    }

    pub fn with_retained_artifacts(mut self, retained_artifacts: Vec<RetainedArtifact>) -> Self {
        self.retained_artifacts = retained_artifacts;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchControllerRequest {
    pub session_id: Option<String>,
    pub plan: SearchPlan,
    pub turns: Vec<SearchTurnRequest>,
    pub state: SearchControllerState,
    pub retained_artifact_limit: usize,
}

impl SearchControllerRequest {
    pub fn new(plan: SearchPlan, turns: Vec<SearchTurnRequest>) -> Self {
        let turn_limit = turns.len();
        Self {
            session_id: None,
            plan,
            turns,
            state: SearchControllerState::new(turn_limit),
            retained_artifact_limit: 5,
        }
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_state(mut self, state: SearchControllerState) -> Self {
        self.state = state;
        self
    }

    pub fn with_retained_artifact_limit(mut self, retained_artifact_limit: usize) -> Self {
        self.retained_artifact_limit = retained_artifact_limit;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SearchControllerAction {
    Retrieve,
    Retain,
    Prune,
    Emit,
    Continue,
    Terminate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchControllerDecision {
    pub action: SearchControllerAction,
    pub rationale: Option<String>,
}

impl SearchControllerDecision {
    pub fn new(action: SearchControllerAction) -> Self {
        Self {
            action,
            rationale: None,
        }
    }

    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = Some(rationale.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchTurn {
    pub session_id: Option<String>,
    pub turn_id: String,
    pub parent_turn_id: Option<String>,
    pub sequence: usize,
    pub path: String,
    pub query: String,
    pub intent: Option<String>,
    pub strategy: String,
    pub limit: usize,
    pub shortlist: usize,
    pub emission_mode: SearchEmissionMode,
    pub result_count: usize,
    pub retained_artifacts: Vec<RetainedArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchTurnTrace {
    pub turn_id: String,
    pub sequence: usize,
    pub query: String,
    pub strategy: String,
    pub emission_mode: SearchEmissionMode,
    pub result_count: usize,
    pub retained_artifacts: Vec<RetainedArtifact>,
    pub decisions: Vec<SearchControllerDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchTrace {
    pub session_id: Option<String>,
    pub turns: Vec<SearchTurnTrace>,
    pub completed: bool,
    pub termination_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtocolSearchEmission {
    pub turn_id: String,
    pub session_id: Option<String>,
    pub strategy: String,
    pub root: String,
    pub hits: Vec<SearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LatentSearchHit {
    pub artifact_id: String,
    pub path: String,
    pub score: f64,
    pub confidence: ScoreConfidence,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LatentSearchEmission {
    pub turn_id: String,
    pub session_id: Option<String>,
    pub feature_space: String,
    pub hits: Vec<LatentSearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "payload", rename_all = "kebab-case")]
pub enum SearchEmission {
    View(SearchResponse),
    Protocol(ProtocolSearchEmission),
    Latent(LatentSearchEmission),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchTurnResponse {
    pub turn: SearchTurn,
    pub assembly: ContextAssemblyResponse,
    pub trace: SearchTrace,
    pub emission: SearchEmission,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchControllerResponse {
    pub plan: SearchPlan,
    pub state: SearchControllerState,
    pub turns: Vec<SearchTurnResponse>,
    pub trace: SearchTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutonomousSearchResponse {
    pub root_task: String,
    pub mode: AutonomousSearchMode,
    pub planner_strategy: AutonomousPlannerStrategy,
    pub plan: SearchPlan,
    pub state: AutonomousPlannerState,
    pub turns: Vec<SearchTurnResponse>,
    pub planner_trace: AutonomousPlannerTrace,
    pub trace: SearchTrace,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextAssemblyBudget {
    pub max_retained_artifacts: usize,
}

impl ContextAssemblyBudget {
    pub fn new(max_retained_artifacts: usize) -> Self {
        Self {
            max_retained_artifacts,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextAssemblyRequest {
    pub path: PathBuf,
    pub query: String,
    pub strategy: Option<String>,
    pub plan: Option<SearchPlan>,
    pub intent: Option<String>,
    pub limit: Option<usize>,
    pub shortlist: Option<usize>,
    pub emission_mode: SearchEmissionMode,
    pub local_context: Vec<LocalContextSource>,
    pub retained_artifacts: Vec<RetainedArtifact>,
    pub budget: ContextAssemblyBudget,
}

impl ContextAssemblyRequest {
    pub fn new(path: impl AsRef<Path>, query: impl Into<String>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            query: query.into(),
            strategy: None,
            plan: None,
            intent: None,
            limit: None,
            shortlist: None,
            emission_mode: SearchEmissionMode::View,
            local_context: Vec::new(),
            retained_artifacts: Vec::new(),
            budget: ContextAssemblyBudget::new(5),
        }
    }

    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.strategy = Some(strategy.into());
        self
    }

    pub fn with_plan(mut self, plan: SearchPlan) -> Self {
        self.plan = Some(plan);
        self
    }

    pub fn with_intent(mut self, intent: impl Into<String>) -> Self {
        self.intent = Some(intent.into());
        self
    }

    pub fn with_intent_opt(mut self, intent: Option<String>) -> Self {
        self.intent = intent;
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

    pub fn with_emission_mode(mut self, emission_mode: SearchEmissionMode) -> Self {
        self.emission_mode = emission_mode;
        self
    }

    pub fn with_local_context(mut self, local_context: Vec<LocalContextSource>) -> Self {
        self.local_context = local_context;
        self
    }

    pub fn with_retained_artifacts(mut self, retained_artifacts: Vec<RetainedArtifact>) -> Self {
        self.retained_artifacts = retained_artifacts;
        self
    }

    pub fn with_budget(mut self, budget: ContextAssemblyBudget) -> Self {
        self.budget = budget;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextAssemblyResponse {
    pub response: SearchResponse,
    pub emission: SearchEmission,
    pub retained_artifacts: Vec<RetainedArtifact>,
    pub pruned_artifacts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedCorpus {
    pub artifacts: Vec<ContextArtifact>,
    pub total_bytes: u64,
    pub indexed_artifacts: usize,
    pub skipped_artifacts: usize,
}

impl LoadedCorpus {
    pub fn artifact_by_id(&self, id: &str) -> Option<&ContextArtifact> {
        self.artifacts.iter().find(|artifact| artifact.id == id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextArtifact {
    pub id: String,
    pub kind: ContextArtifactKind,
    pub path: PathBuf,
    pub source_kind: SourceKind,
    pub length: usize,
    pub terms: HashMap<String, usize>,
    pub text: String,
    pub segments: Vec<Segment>,
    pub provenance: ArtifactProvenance,
    pub freshness: ArtifactFreshness,
    pub budget: ArtifactBudget,
}

impl ContextArtifact {
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bm25Index {
    pub doc_freq: HashMap<String, usize>,
    pub term_freqs: HashMap<String, HashMap<String, usize>>,
    pub doc_lengths: HashMap<String, usize>,
    pub avg_doc_len: f64,
    pub num_docs: usize,
}

impl Bm25Index {
    pub fn build(artifacts: &[ContextArtifact]) -> Self {
        let mut doc_freq = HashMap::new();
        let mut term_freqs = HashMap::new();
        let mut doc_lengths = HashMap::new();
        let mut total_len = 0;

        for doc in artifacts {
            let terms: HashSet<_> = doc.terms.keys().collect();
            term_freqs.insert(doc.id.clone(), doc.terms.clone());
            doc_lengths.insert(doc.id.clone(), doc.length);
            total_len += doc.length;

            for term in terms {
                *doc_freq.entry(term.clone()).or_insert(0) += 1;
            }
        }

        let avg_doc_len = if artifacts.is_empty() {
            0.0
        } else {
            total_len as f64 / artifacts.len() as f64
        };

        Self {
            doc_freq,
            term_freqs,
            doc_lengths,
            avg_doc_len,
            num_docs: artifacts.len(),
        }
    }

    pub fn score(&self, query: &[String]) -> Vec<(String, f64)> {
        let mut scores = HashMap::new();
        let k1 = 1.2;
        let b = 0.75;

        for term in query {
            if let Some(&df) = self.doc_freq.get(term) {
                let idf = ((self.num_docs as f64 - df as f64 + 0.5) / (df as f64 + 0.5) + 1.0).ln();
                for (artifact_id, terms) in &self.term_freqs {
                    let Some(&tf) = terms.get(term) else {
                        continue;
                    };
                    let doc_len = *self.doc_lengths.get(artifact_id).unwrap_or(&0) as f64;
                    let tf = tf as f64;
                    let score = idf * (tf * (k1 + 1.0))
                        / (tf + k1 * (1.0 - b + b * doc_len / self.avg_doc_len));
                    *scores.entry(artifact_id.clone()).or_insert(0.0) += score;
                }
            }
        }
        let mut results: Vec<_> = scores.into_iter().collect();
        results.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        results
    }
}

pub struct StrategyPreset {
    pub name: String,
    pub plan: SearchPlan,
}

pub struct StrategyPresetRegistry {
    presets: HashMap<String, SearchPlan>,
}

impl Default for StrategyPresetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyPresetRegistry {
    pub fn new() -> Self {
        Self {
            presets: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, plan: SearchPlan) {
        self.presets.insert(name.to_string(), plan);
    }

    pub fn resolve(&self, name: &str) -> Result<SearchPlan> {
        self.presets
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow!("strategy not found: {}", name))
    }

    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.presets.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn default_registry() -> Self {
        let mut registry = Self::new();

        let lexical_plan = SearchPlan {
            name: "lexical".to_string(),
            query_expansion: QueryExpansionPolicy::None,
            retrievers: vec![RetrieverPolicy::Bm25],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::None,
        };
        registry.register("lexical", lexical_plan.clone());
        registry.register(
            "bm25",
            SearchPlan {
                name: "bm25".to_string(),
                ..lexical_plan.clone()
            },
        );

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

        let hybrid_plan = SearchPlan {
            name: "hybrid".to_string(),
            query_expansion: QueryExpansionPolicy::None,
            retrievers: vec![RetrieverPolicy::Bm25, RetrieverPolicy::Vector],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::None,
        };
        registry.register("hybrid", hybrid_plan);

        let page_index_hybrid_plan = SearchPlan {
            name: "page-index-hybrid".to_string(),
            query_expansion: QueryExpansionPolicy::Splade,
            retrievers: vec![
                RetrieverPolicy::Bm25,
                RetrieverPolicy::Phrase,
                RetrieverPolicy::Vector,
            ],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::PositionAware,
        };
        registry.register("page-index-hybrid", page_index_hybrid_plan.clone());
        registry.register(
            "legacy-hybrid",
            SearchPlan {
                name: "legacy-hybrid".to_string(),
                query_expansion: QueryExpansionPolicy::None,
                retrievers: page_index_hybrid_plan.retrievers.clone(),
                fusion: page_index_hybrid_plan.fusion,
                reranking: page_index_hybrid_plan.reranking,
            },
        );

        let page_index_llm_plan = SearchPlan {
            name: "page-index-llm".to_string(),
            query_expansion: QueryExpansionPolicy::Hyde,
            retrievers: vec![
                RetrieverPolicy::Bm25,
                RetrieverPolicy::Phrase,
                RetrieverPolicy::Vector,
            ],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::Llm,
        };
        registry.register("page-index-llm", page_index_llm_plan);

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

        // page-index-gemma (Gemma 3)
        registry.register(
            "page-index-gemma",
            SearchPlan {
                name: "page-index-gemma".to_string(),
                query_expansion: QueryExpansionPolicy::Splade,
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                    RetrieverPolicy::Vector,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::Gemma,
            },
        );

        registry
    }
}

pub trait Retriever: Send + Sync {
    fn retrieve(
        &self,
        query: &[QueryVariant],
        corpus: &PreparedCorpus,
        limit: usize,
        verbose: u8,
    ) -> Result<CandidateList>;
    fn policy(&self) -> RetrieverPolicy;
}

pub trait Fuser: Send + Sync {
    fn fuse(&self, lists: &[CandidateList], limit: usize, verbose: u8) -> Result<CandidateList>;
}

pub trait Expander: Send + Sync {
    fn expand(&self, query: &str) -> Vec<QueryVariant>;
}

pub trait Reranker: Send + Sync {
    fn rerank(&self, query: &str, candidates: CandidateList, limit: usize)
    -> Result<CandidateList>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_generative(&self) -> Option<&dyn GenerativeModel> {
        None
    }
}

pub trait Conversation: Send + Sync {
    fn send(&mut self, message: &str, max_tokens: usize) -> Result<String>;
    fn history(&self) -> &[String];
}

pub trait GenerativeModel: Send + Sync {
    fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String>;
    fn start_conversation(&self) -> Result<Box<dyn Conversation>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateList {
    pub results: Vec<Candidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub path: std::path::PathBuf,
    pub score: f64,
    pub contributors: Vec<ContributorScore>,
    pub snippet: Option<String>,
    pub snippet_location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContributorScore {
    pub retriever: RetrieverPolicy,
    pub score: f64,
}

pub trait Embedder: Send + Sync {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
}

#[derive(Clone)]
pub struct CachedEmbedder {
    pub inner: Arc<dyn Embedder>,
    pub cache: QueryEmbeddingCache,
}

impl Embedder for CachedEmbedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut cached = Vec::with_capacity(texts.len());
        let mut missing_indices = Vec::new();
        let mut missing_texts = Vec::new();

        {
            let cache = self
                .cache
                .read()
                .map_err(|_| anyhow!("query embedding cache read lock poisoned"))?;

            for (index, text) in texts.iter().enumerate() {
                if let Some(embedding) = cache.get(text) {
                    cached.push(Some(embedding.clone()));
                } else {
                    cached.push(None);
                    missing_indices.push(index);
                    missing_texts.push(text.clone());
                }
            }
        }

        if !missing_texts.is_empty() {
            let computed = self.inner.embed(&missing_texts)?;
            if computed.len() != missing_texts.len() {
                return Err(anyhow!(
                    "embedder returned {} embeddings for {} inputs",
                    computed.len(),
                    missing_texts.len()
                ));
            }

            let mut cache = self
                .cache
                .write()
                .map_err(|_| anyhow!("query embedding cache write lock poisoned"))?;

            for (result_offset, original_index) in missing_indices.into_iter().enumerate() {
                let embedding = computed[result_offset].clone();
                let text = texts[original_index].clone();
                cache.insert(text, embedding.clone());
                cached[original_index] = Some(embedding);
            }
        }

        cached
            .into_iter()
            .map(|embedding| {
                embedding.ok_or_else(|| anyhow!("missing embedding after cache resolution"))
            })
            .collect()
    }

    fn dimension(&self) -> usize {
        self.inner.dimension()
    }
}

pub type QueryEmbeddingCache = Arc<std::sync::RwLock<HashMap<String, Vec<f32>>>>;

pub struct CorpusLoadRequest<'a> {
    pub path: &'a std::path::Path,
    pub ignore: Option<&'a Ignore>,
    pub verbose: u8,
    pub embedder: Option<&'a dyn Embedder>,
    pub telemetry: &'a crate::system::Telemetry,
    pub local_context: &'a [LocalContextSource],
    pub cache_dir: Option<&'a std::path::Path>,
}

pub trait CorpusRepository: Send + Sync {
    fn load(&self, request: &CorpusLoadRequest<'_>) -> Result<LoadedCorpus>;

    fn load_with_progress(
        &self,
        request: &CorpusLoadRequest<'_>,
        progress: Option<&dyn Fn(&SearchProgress)>,
    ) -> Result<LoadedCorpus> {
        let _ = progress;
        self.load(request)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchTelemetry {
    pub heuristic_hits: usize,
    pub blob_hits: usize,
    pub fresh_artifact_builds: usize,
    pub skipped_artifacts: usize,
    pub embedding_hits: usize,
    pub total_files: usize,
    pub total_segments: usize,
    pub bm25_index_cache_hits: usize,
    pub bm25_index_builds: usize,
    pub sector_cache_hits: usize,
    pub sector_rebuilds: usize,
    pub sector_shard_cache_hits: usize,
    pub sector_shard_builds: usize,
}

impl SearchTelemetry {
    pub fn capture(telemetry: &crate::system::Telemetry) -> Self {
        Self {
            heuristic_hits: telemetry.heuristic_hits.load(Ordering::Relaxed),
            blob_hits: telemetry.blob_hits.load(Ordering::Relaxed),
            fresh_artifact_builds: telemetry.fresh_artifact_builds.load(Ordering::Relaxed),
            skipped_artifacts: telemetry.skipped_artifacts.load(Ordering::Relaxed),
            embedding_hits: telemetry.embedding_hits.load(Ordering::Relaxed),
            total_files: telemetry.total_files.load(Ordering::Relaxed),
            total_segments: telemetry.total_segments.load(Ordering::Relaxed),
            bm25_index_cache_hits: telemetry.bm25_index_cache_hits.load(Ordering::Relaxed),
            bm25_index_builds: telemetry.bm25_index_builds.load(Ordering::Relaxed),
            sector_cache_hits: telemetry.sector_cache_hits.load(Ordering::Relaxed),
            sector_rebuilds: telemetry.sector_rebuilds.load(Ordering::Relaxed),
            sector_shard_cache_hits: telemetry.sector_shard_cache_hits.load(Ordering::Relaxed),
            sector_shard_builds: telemetry.sector_shard_builds.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryVariant {
    pub text: String,
    pub weight: f64,
}

pub struct PreparedCorpus<'a> {
    pub artifacts: &'a [ContextArtifact],
    pub bm25_index: Option<&'a Bm25Index>,
}

pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    Text,
}

#[cfg(test)]
mod tests {
    use super::{QueryExpansionPolicy, RerankingPolicy, RetrieverPolicy, StrategyPresetRegistry};

    #[test]
    fn default_registry_includes_vector_strategy() {
        let plan = StrategyPresetRegistry::default_registry()
            .resolve("vector")
            .expect("vector preset should be registered");

        assert_eq!(plan.name, "vector");
        assert_eq!(plan.query_expansion, QueryExpansionPolicy::None);
        assert_eq!(plan.retrievers, vec![RetrieverPolicy::Vector]);
        assert_eq!(plan.reranking, RerankingPolicy::None);
    }
}
