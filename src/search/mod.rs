pub mod adapters;
pub mod application;
pub mod corpus;
pub mod domain;
pub mod engine;
pub mod legacy;
pub mod presentation;

pub use adapters::render_search_response;
pub use application::{
    LocalFileCorpusRepository, SearchService, SearchServiceBuilder, run_search,
    run_search_with_plan,
};
pub use corpus::load_search_corpus;
pub use domain::{
    AcquisitionAdapterKind, AgentTurnInput, ArtifactBudget, ArtifactFreshness, ArtifactProvenance,
    AutonomousPlannerState, AutonomousPlannerStepCursor, AutonomousPlannerStrategy,
    AutonomousPlannerStrategyKind, AutonomousSearchRequest, AutonomousSearchResponse, Bm25Index,
    CachedEmbedder, Candidate, CandidateList, ContextArtifact, ContextArtifactKind,
    ContextAssemblyBudget, ContextAssemblyRequest, ContextAssemblyResponse, ContributorScore,
    Conversation, CorpusLoadRequest, CorpusRepository, Embedder, EnvironmentFactInput, Expander,
    Fuser, FusionPolicy, GenerativeModel, LatentSearchEmission, LatentSearchHit, LoadedCorpus,
    LocalContextSource, OutputFormat, PreparedCorpus, ProtocolSearchEmission, QueryEmbeddingCache,
    QueryExpansionPolicy, QueryVariant, Reranker, RerankingPolicy, RetainedArtifact, Retriever,
    RetrieverPolicy, ScoreConfidence, SearchControllerAction, SearchControllerDecision,
    SearchControllerRequest, SearchControllerResponse, SearchControllerState, SearchEmission,
    SearchEmissionMode, SearchHit, SearchPlan, SearchRequest, SearchResponse, SearchTelemetry,
    SearchTrace, SearchTurn, SearchTurnRequest, SearchTurnResponse, SearchTurnTrace,
    StrategyPresetRegistry, ToolOutputInput, tokenize,
};
pub use engine::{SearchEngine, SearchEnvironment};
#[allow(unused_imports)]
pub use presentation::*;
