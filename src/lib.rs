pub mod cache;
pub mod config;
pub mod dense;
pub mod eval;
pub mod extract;
mod facade;
pub mod hybrid;
pub mod search;
pub mod segment;
pub mod system;
pub mod vector;

pub use crate::facade::{
    Fusion, Reranking, Retriever, SearchInput, SearchOptions, Sift, SiftBuilder,
};
pub use crate::search::{ScoreConfidence, SearchHit, SearchResponse};
