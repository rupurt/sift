use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;

use crate::segment::Segment;

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentHit {
    pub segment_id: String,
    pub doc_id: String,
    pub path: PathBuf,
    pub label: String,
    pub text: String,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticDocumentHit {
    pub id: String,
    pub path: PathBuf,
    pub score: f64,
    pub best_segment_id: String,
    pub best_segment_label: String,
    pub best_segment_text: String,
    pub best_segment_score: f64,
    pub segment_hits: usize,
}

pub fn score_segments_manually(
    scorer: &crate::dense::DenseReranker,
    query: &str,
    segments: &[Segment],
) -> Result<Vec<SegmentHit>> {
    if segments.is_empty() {
        return Ok(Vec::new());
    }

    // Embed the query
    let query_embeddings = scorer.embed_batch(&[query.to_string()])?;
    let query_vec = &query_embeddings[0];

    let mut hits = Vec::with_capacity(segments.len());
    for segment in segments {
        if let Some(doc_vec) = &segment.embedding {
            let score = dot_product(query_vec, doc_vec);
            hits.push(SegmentHit {
                segment_id: segment.id.clone(),
                doc_id: segment.doc_id.clone(),
                path: segment.path.clone(),
                label: segment.label.clone(),
                text: segment.text.clone(),
                score: score as f64,
            });
        }
    }

    Ok(hits)
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

pub trait SegmentScorer {
    fn score_segments(&self, query: &str, segments: &[Segment]) -> Result<Vec<SegmentHit>>;
}

pub fn retrieve_semantic_documents<S: SegmentScorer>(
    scorer: &S,
    query: &str,
    segments: &[Segment],
    limit: usize,
) -> Result<Vec<SemanticDocumentHit>> {
    if segments.is_empty() {
        return Ok(Vec::new());
    }

    let hits = scorer
        .score_segments(query, segments)?
        .into_iter()
        .filter(|hit| hit.score.is_finite() && hit.score > 0.0)
        .collect::<Vec<_>>();

    Ok(aggregate_segment_hits(&hits)
        .into_iter()
        .take(limit.max(1))
        .collect())
}

pub fn aggregate_segment_hits(hits: &[SegmentHit]) -> Vec<SemanticDocumentHit> {
    #[derive(Debug, Clone)]
    struct DocAccumulator {
        id: String,
        path: PathBuf,
        total_score: f64,
        segment_hits: usize,
        best_segment: SegmentHit,
    }

    let mut documents = HashMap::<String, DocAccumulator>::new();

    for hit in hits {
        let mut doc_id = hit.doc_id.clone();
        if doc_id.starts_with("./") {
            doc_id = doc_id.chars().skip(2).collect();
        }
        let entry = documents
            .entry(doc_id.clone())
            .or_insert_with(|| DocAccumulator {
                id: doc_id,
                path: hit.path.clone(),
                total_score: 0.0,
                segment_hits: 0,
                best_segment: hit.clone(),
            });
        entry.total_score += hit.score;
        entry.segment_hits += 1;

        let replace_best = hit.score > entry.best_segment.score
            || (hit.score == entry.best_segment.score
                && hit.segment_id < entry.best_segment.segment_id);
        if replace_best {
            entry.best_segment = hit.clone();
        }
    }

    let mut ranked = documents
        .into_values()
        .map(|document| SemanticDocumentHit {
            id: document.id,
            path: document.path,
            score: document.total_score / ((document.segment_hits as f64 + 1.0).sqrt()),
            best_segment_id: document.best_segment.segment_id,
            best_segment_label: document.best_segment.label,
            best_segment_text: document.best_segment.text,
            best_segment_score: document.best_segment.score,
            segment_hits: document.segment_hits,
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.id.cmp(&right.id))
    });
    ranked
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::path::PathBuf;

    use anyhow::Result;

    use super::{SegmentHit, SegmentScorer, aggregate_segment_hits, retrieve_semantic_documents};
    use crate::extract::SourceKind;
    use crate::segment::Segment;

    mod vector_retrieval {
        use super::*;

        mod full_corpus {
            use super::*;

            #[test]
            fn scores_the_full_active_segment_corpus() {
                let segments = vec![
                    sample_segment("doc-a::segment:0001", "doc-a", "alpha.txt", "alpha body"),
                    sample_segment("doc-b::segment:0001", "doc-b", "beta.txt", "beta body"),
                    sample_segment("doc-c::segment:0001", "doc-c", "gamma.txt", "semantic body"),
                ];
                let expected_ids = segments
                    .iter()
                    .map(|segment| segment.id.clone())
                    .collect::<Vec<_>>();
                let scorer = RecordingScorer {
                    seen_segment_ids: RefCell::new(Vec::new()),
                    scores: HashMap::from([
                        (segments[0].id.clone(), 0.2),
                        (segments[1].id.clone(), 0.1),
                        (segments[2].id.clone(), 0.9),
                    ]),
                };

                let ranked =
                    retrieve_semantic_documents(&scorer, "semantic retrieval", &segments, 10)
                        .expect("semantic ranking");

                assert_eq!(*scorer.seen_segment_ids.borrow(), expected_ids);
                assert_eq!(ranked[0].id, "doc-c");
            }
        }

        mod aggregation {
            use super::*;

            #[test]
            fn uses_the_planned_diminishing_returns_rule() {
                let hits = vec![
                    sample_hit("doc-a::segment:0001", "doc-a", "alpha.txt", 0.9),
                    sample_hit("doc-a::segment:0002", "doc-a", "alpha.txt", 0.4),
                    sample_hit("doc-b::segment:0001", "doc-b", "beta.txt", 1.0),
                ];

                let ranked = aggregate_segment_hits(&hits);

                assert_eq!(ranked[0].id, "doc-a");
                assert!((ranked[0].score - (1.3 / 3_f64.sqrt())).abs() < 1.0e-9);
                assert_eq!(ranked[0].best_segment_id, "doc-a::segment:0001");
                assert_eq!(ranked[0].segment_hits, 2);
                assert_eq!(ranked[1].id, "doc-b");
                assert!((ranked[1].score - (1.0 / 2_f64.sqrt())).abs() < 1.0e-9);
            }
        }
    }

    struct RecordingScorer {
        seen_segment_ids: RefCell<Vec<String>>,
        scores: HashMap<String, f64>,
    }

    impl SegmentScorer for RecordingScorer {
        fn score_segments(&self, _query: &str, segments: &[Segment]) -> Result<Vec<SegmentHit>> {
            self.seen_segment_ids.replace(
                segments
                    .iter()
                    .map(|segment| segment.id.clone())
                    .collect::<Vec<_>>(),
            );

            Ok(segments
                .iter()
                .map(|segment| SegmentHit {
                    segment_id: segment.id.clone(),
                    doc_id: segment.doc_id.clone(),
                    path: segment.path.clone(),
                    label: segment.label.clone(),
                    text: segment.text.clone(),
                    score: *self.scores.get(&segment.id).unwrap_or(&0.0),
                })
                .collect())
        }
    }

    fn sample_segment(id: &str, doc_id: &str, file_name: &str, text: &str) -> Segment {
        Segment {
            id: id.to_string(),
            doc_id: doc_id.to_string(),
            path: PathBuf::from(file_name),
            source_kind: SourceKind::Text,
            ordinal: 1,
            label: "section 1".to_string(),
            text: text.to_string(),
            embedding: None,
        }
    }

    fn sample_hit(segment_id: &str, doc_id: &str, file_name: &str, score: f64) -> SegmentHit {
        SegmentHit {
            segment_id: segment_id.to_string(),
            doc_id: doc_id.to_string(),
            path: PathBuf::from(file_name),
            label: "section 1".to_string(),
            text: format!("sample text for {segment_id}"),
            score,
        }
    }
}
