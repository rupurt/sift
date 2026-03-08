use std::cmp::Ordering;

use anyhow::Result;

const BM25_WEIGHT: f64 = 0.35;
const DENSE_WEIGHT: f64 = 0.65;
const BM25_TIE_BONUS_SCALE: f64 = 1000.0;

#[derive(Debug, Clone, PartialEq)]
pub struct HybridCandidate {
    pub id: String,
    pub bm25_rank: usize,
    pub bm25_score: f64,
    pub dense_score: f64,
    pub final_score: f64,
}

pub fn fuse_candidates(candidates: &[HybridCandidate]) -> Result<Vec<HybridCandidate>> {
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let (bm25_min, bm25_max) =
        score_bounds(candidates.iter().map(|candidate| candidate.bm25_score));
    let (dense_min, dense_max) =
        score_bounds(candidates.iter().map(|candidate| candidate.dense_score));

    let mut fused = candidates
        .iter()
        .cloned()
        .map(|mut candidate| {
            let lexical = normalize(candidate.bm25_score, bm25_min, bm25_max);
            let semantic = normalize(candidate.dense_score, dense_min, dense_max);
            let tie_bonus = 1.0 / (BM25_TIE_BONUS_SCALE + candidate.bm25_rank as f64);
            candidate.final_score = lexical * BM25_WEIGHT + semantic * DENSE_WEIGHT + tie_bonus;
            candidate
        })
        .collect::<Vec<_>>();

    fused.sort_by(|left, right| {
        right
            .final_score
            .partial_cmp(&left.final_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.bm25_rank.cmp(&right.bm25_rank))
            .then_with(|| left.id.cmp(&right.id))
    });

    Ok(fused)
}

fn score_bounds(scores: impl Iterator<Item = f64>) -> (f64, f64) {
    let mut min_score = f64::INFINITY;
    let mut max_score = f64::NEG_INFINITY;

    for score in scores {
        min_score = min_score.min(score);
        max_score = max_score.max(score);
    }

    if !min_score.is_finite() || !max_score.is_finite() {
        (0.0, 0.0)
    } else {
        (min_score, max_score)
    }
}

fn normalize(score: f64, min_score: f64, max_score: f64) -> f64 {
    let range = max_score - min_score;
    if range.abs() < f64::EPSILON {
        1.0
    } else {
        (score - min_score) / range
    }
}

#[cfg(test)]
mod fusion {
    use super::{HybridCandidate, fuse_candidates};

    #[test]
    fn dense_signal_can_reorder_bm25_shortlist() {
        let fused = fuse_candidates(&[
            HybridCandidate {
                id: "doc-a".to_string(),
                bm25_rank: 1,
                bm25_score: 12.0,
                dense_score: 0.10,
                final_score: 0.0,
            },
            HybridCandidate {
                id: "doc-b".to_string(),
                bm25_rank: 2,
                bm25_score: 11.0,
                dense_score: 0.95,
                final_score: 0.0,
            },
        ])
        .expect("fused candidates");

        assert_eq!(fused[0].id, "doc-b");
        assert_eq!(fused[1].id, "doc-a");
        assert!(fused[0].final_score > fused[1].final_score);
    }
}
