//! `sift` supports embedding through the crate root.
//!
//! Supported embedded API:
//! - [`Sift`] and [`SiftBuilder`] for constructing a search engine instance
//! - [`SearchInput`] and [`SearchOptions`] for issuing searches
//! - [`Retriever`], [`Fusion`], and [`Reranking`] for supported strategy overrides
//! - [`SearchResponse`], [`SearchHit`], and [`ScoreConfidence`] for results
//!
//! Everything under [`internal`] exists to support the bundled executable,
//! benchmarks, and repository-internal tests. It is not part of the supported
//! embedding contract and may change without notice.

mod cache;
mod config;
mod dense;
mod eval;
mod extract;
mod facade;
mod hybrid;
mod search;
mod segment;
mod system;
mod vector;

pub use crate::facade::{
    Fusion, Reranking, Retriever, SearchInput, SearchOptions, Sift, SiftBuilder,
};
pub use crate::search::{ScoreConfidence, SearchHit, SearchResponse};

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

    pub mod search {
        pub use crate::search::*;

        pub mod adapters {
            pub use crate::search::adapters::*;

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
