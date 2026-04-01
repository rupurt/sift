pub mod adapters;
pub mod application;
pub mod corpus;
pub mod domain;
pub mod engine;
pub mod graph;
pub mod legacy;
pub mod planner;
pub mod presentation;

pub use adapters::{render_autonomous_search_response, render_search_response};
pub use application::{
    LocalFileCorpusRepository, SearchService, SearchServiceBuilder, run_search,
    run_search_with_plan,
};
pub use corpus::{load_search_corpus, load_search_corpus_with_progress};
pub use domain::{
    AcquisitionAdapterKind, AgentTurnInput, ArtifactBudget, ArtifactFreshness, ArtifactProvenance,
    AutonomousGraphBranchState, AutonomousGraphBranchStatus, AutonomousGraphEdge,
    AutonomousGraphEdgeKind, AutonomousGraphEpisodeState, AutonomousGraphFrontierEntry,
    AutonomousGraphNode, AutonomousPlanner, AutonomousPlannerAction, AutonomousPlannerDecision,
    AutonomousPlannerState, AutonomousPlannerStepCursor, AutonomousPlannerStopReason,
    AutonomousPlannerStrategy, AutonomousPlannerStrategyKind, AutonomousPlannerTrace,
    AutonomousPlannerTraceStep, AutonomousSearchMode, AutonomousSearchRequest,
    AutonomousSearchResponse, Bm25Index, CachedEmbedder, Candidate, CandidateList, ContextArtifact,
    ContextArtifactKind, ContextAssemblyBudget, ContextAssemblyRequest, ContextAssemblyResponse,
    ContributorScore, Conversation, CorpusLoadRequest, CorpusRepository, Embedder,
    EnvironmentFactInput, Expander, Fuser, FusionPolicy, GenerativeModel, LatentSearchEmission,
    LatentSearchHit, LoadedCorpus, LocalContextSource, OutputFormat, PreparedCorpus,
    ProtocolSearchEmission, QueryEmbeddingCache, QueryExpansionPolicy, QueryVariant, Reranker,
    RerankingPolicy, RetainedArtifact, Retriever, RetrieverPolicy, ScoreConfidence,
    SearchControllerAction, SearchControllerDecision, SearchControllerRequest,
    SearchControllerResponse, SearchControllerState, SearchEmission, SearchEmissionMode, SearchHit,
    SearchPhase, SearchPlan, SearchProgress, SearchRequest, SearchResponse, SearchTelemetry,
    SearchTrace, SearchTurn, SearchTurnRequest, SearchTurnResponse, SearchTurnTrace,
    StrategyPresetRegistry, ToolOutputInput, tokenize,
};
pub use engine::{SearchEngine, SearchEnvironment};
pub use graph::{
    AutonomousGraphTraceContractError, AutonomousGraphTraceContractErrorKind,
    replay_graph_decision, replay_graph_trace,
};
pub use planner::{HeuristicAutonomousPlanner, ModelDrivenAutonomousPlanner};
#[allow(unused_imports)]
pub use presentation::*;
