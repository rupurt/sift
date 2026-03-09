use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;

use crate::vector::SemanticDocumentHit;

const RRF_K: usize = 60;

#[derive(Debug, Clone, PartialEq)]
pub struct Bm25DocumentHit {
    pub id: String,
    pub path: PathBuf,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HybridDocumentHit {
    pub id: String,
    pub path: PathBuf,
    pub score: f64,
    pub bm25_score: Option<f64>,
    pub semantic_score: Option<f64>,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone)]
struct HybridAccumulator {
    id: String,
    path: PathBuf,
    score: f64,
    bm25_score: Option<f64>,
    semantic_score: Option<f64>,
    snippet: Option<String>,
}

pub fn fuse_rankings(
    bm25: &[Bm25DocumentHit],
    semantic: &[SemanticDocumentHit],
    limit: usize,
) -> Result<Vec<HybridDocumentHit>> {
    let mut documents = HashMap::<String, HybridAccumulator>::new();

    for (index, document) in bm25.iter().enumerate() {
        let entry = documents
            .entry(document.id.clone())
            .or_insert_with(|| HybridAccumulator {
                id: document.id.clone(),
                path: document.path.clone(),
                score: 0.0,
                bm25_score: None,
                semantic_score: None,
                snippet: None,
            });
        entry.score += reciprocal_rank(index + 1);
        entry.bm25_score = Some(document.score);
    }

    for (index, document) in semantic.iter().enumerate() {
        let entry = documents
            .entry(document.id.clone())
            .or_insert_with(|| HybridAccumulator {
                id: document.id.clone(),
                path: document.path.clone(),
                score: 0.0,
                bm25_score: None,
                semantic_score: None,
                snippet: None,
            });
        entry.score += reciprocal_rank(index + 1);
        entry.path = document.path.clone();
        entry.semantic_score = Some(document.score);
        entry.snippet = Some(document.best_segment_text.clone());
    }

    let mut fused = documents
        .into_values()
        .map(|document| HybridDocumentHit {
            id: document.id,
            path: document.path,
            score: document.score,
            bm25_score: document.bm25_score,
            semantic_score: document.semantic_score,
            snippet: document.snippet,
        })
        .collect::<Vec<_>>();

    fused.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.id.cmp(&right.id))
    });

    Ok(fused.into_iter().take(limit.max(1)).collect())
}

fn reciprocal_rank(rank: usize) -> f64 {
    1.0 / (RRF_K as f64 + rank as f64)
}

#[cfg(test)]
mod rrf {
    use std::path::PathBuf;

    use crate::vector::SemanticDocumentHit;

    use super::{Bm25DocumentHit, fuse_rankings};

    #[test]
    fn fuses_independent_bm25_and_vector_rankings() {
        let fused = fuse_rankings(
            &[
                Bm25DocumentHit {
                    id: "doc-a".to_string(),
                    path: PathBuf::from("alpha.txt"),
                    score: 12.0,
                },
                Bm25DocumentHit {
                    id: "doc-b".to_string(),
                    path: PathBuf::from("beta.txt"),
                    score: 11.0,
                },
            ],
            &[
                SemanticDocumentHit {
                    id: "doc-b".to_string(),
                    path: PathBuf::from("beta.txt"),
                    score: 0.9,
                    best_segment_id: "doc-b::segment:0001".to_string(),
                    best_segment_label: "section 1".to_string(),
                    best_segment_text: "semantic beta snippet".to_string(),
                    best_segment_score: 0.9,
                    segment_hits: 1,
                },
                SemanticDocumentHit {
                    id: "doc-c".to_string(),
                    path: PathBuf::from("gamma.txt"),
                    score: 0.8,
                    best_segment_id: "doc-c::segment:0001".to_string(),
                    best_segment_label: "section 1".to_string(),
                    best_segment_text: "semantic gamma snippet".to_string(),
                    best_segment_score: 0.8,
                    segment_hits: 1,
                },
            ],
            10,
        )
        .expect("fused rankings");

        assert_eq!(fused[0].id, "doc-b");
        assert!(fused[0].score > fused[1].score);
        assert_eq!(fused[0].snippet.as_deref(), Some("semantic beta snippet"));
    }
}
