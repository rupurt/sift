//! `sift` supports embedding through the crate root.
//!
//! Supported embedded API:
//! - [`Sift`] and [`SiftBuilder`] for constructing a search engine instance
//! - [`SearchInput`] and [`SearchOptions`] for issuing direct searches
//! - [`ContextAssemblyRequest`] and [`ContextAssemblyResponse`] for bounded evidence assembly
//! - [`SearchTurnRequest`], [`SearchTurnResponse`], and [`SearchEmissionMode`] for turn-aware search control
//! - [`SearchControllerRequest`] and [`SearchControllerResponse`] for deterministic multi-turn control
//! - [`AutonomousSearchRequest`] and [`AutonomousSearchResponse`] for linear autonomous-planner contracts
//! - [`AutonomousPlanner`] and [`AutonomousPlannerTrace`] for the library-first autonomous execution seam
//! - [`Retriever`], [`Fusion`], and [`Reranking`] for supported strategy overrides
//! - [`SearchResponse`], [`SearchHit`], [`ContextArtifact`], and [`ScoreConfidence`] for results and artifact metadata
//!
//! Everything under [`internal`] exists to support the bundled executable,
//! benchmarks, and repository-internal tests. It is not part of the supported
//! embedding contract and may change without notice.
//!
//! Repository-level usage notes for all supported modes live in `LIBRARY.md`.

mod cache;
mod config;
mod dense;
mod eval;
mod extract;
mod facade;
mod hybrid;
mod optimize;
mod search;
mod segment;
mod system;
mod vector;

pub use crate::facade::{
    Conversation, Fusion, GenerativeModel, Reranking, Retriever, SearchInput, SearchOptions, Sift,
    SiftBuilder,
};
pub use crate::search::{
    AcquisitionAdapterKind, AgentTurnInput, ArtifactBudget, ArtifactFreshness, ArtifactProvenance,
    AutonomousPlanner, AutonomousPlannerAction, AutonomousPlannerDecision, AutonomousPlannerState,
    AutonomousPlannerStepCursor, AutonomousPlannerStopReason, AutonomousPlannerStrategy,
    AutonomousPlannerStrategyKind, AutonomousPlannerTrace, AutonomousPlannerTraceStep,
    AutonomousSearchRequest, AutonomousSearchResponse, ContextArtifact, ContextArtifactKind,
    ContextAssemblyBudget, ContextAssemblyRequest, ContextAssemblyResponse, CorpusLoadRequest,
    EnvironmentFactInput, FusionPolicy, LatentSearchEmission, LatentSearchHit, LocalContextSource,
    ProtocolSearchEmission, QueryExpansionPolicy, RerankingPolicy, RetainedArtifact,
    RetrieverPolicy, ScoreConfidence, SearchControllerAction, SearchControllerDecision,
    SearchControllerRequest, SearchControllerResponse, SearchControllerState, SearchEmission,
    SearchEmissionMode, SearchHit, SearchPlan, SearchResponse, SearchTrace, SearchTurn,
    SearchTurnRequest, SearchTurnResponse, SearchTurnTrace, ToolOutputInput,
};

#[doc(hidden)]
pub mod internal {
    pub mod cache {
        pub use crate::cache::*;
    }

    pub mod config {
        pub use crate::config::*;
    }

    pub mod dense {
        pub use crate::dense::*;
    }

    pub mod eval {
        pub use crate::eval::*;
    }

    pub mod extract {
        pub use crate::extract::*;
    }

    pub mod hybrid {
        pub use crate::hybrid::*;
    }

    pub mod optimize {
        pub use crate::optimize::*;
    }

    pub mod search {
        pub use crate::search::*;

        pub mod adapters {
            pub use crate::search::adapters::*;

            pub mod gemma {
                pub use crate::search::adapters::gemma::*;
            }

            pub mod qwen {
                pub use crate::search::adapters::qwen::*;
            }
        }
    }

    pub mod segment {
        pub use crate::segment::*;
    }

    pub mod system {
        pub use crate::system::*;
    }

    pub mod vector {
        pub use crate::vector::*;
    }
}
