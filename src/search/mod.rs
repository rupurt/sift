pub mod adapters;
pub mod application;
pub mod corpus;
pub mod domain;
pub mod engine;
pub mod legacy;
pub mod presentation;

pub use adapters::render_search_response;
pub use application::{LocalFileCorpusRepository, SearchService, SearchServiceBuilder, run_search};
pub use corpus::load_search_corpus;
pub use domain::{
    Bm25Index, CachedEmbedder, Candidate, CandidateList, ContributorScore, Conversation,
    CorpusRepository, Document, Embedder, Expander, Fuser, FusionPolicy, GenerativeModel,
    LatentSearchEmission, LatentSearchHit, LoadedCorpus, OutputFormat, PreparedCorpus,
    ProtocolSearchEmission, QueryEmbeddingCache, QueryVariant, Reranker, RerankingPolicy,
    RetainedEvidence, Retriever, RetrieverPolicy, ScoreConfidence, SearchControllerAction,
    SearchControllerDecision, SearchEmission, SearchEmissionMode, SearchHit, SearchPlan,
    SearchRequest, SearchResponse, SearchTelemetry, SearchTrace, SearchTurn, SearchTurnRequest,
    SearchTurnResponse, SearchTurnTrace, StrategyPresetRegistry, tokenize,
};
pub use engine::{SearchEngine, SearchEnvironment};
#[allow(unused_imports)]
pub use presentation::*;
