use super::domain::*;
use crate::dense::DenseReranker;
use anyhow::Result;

use crate::vector::{aggregate_segment_hits, score_segments_manually, SegmentScorer};

pub struct SegmentVectorRetriever {
    pub dense: DenseReranker,
}

impl Retriever for SegmentVectorRetriever {
    fn retrieve(
        &self,
        query_variants: &[QueryVariant],
        corpus: &PreparedCorpus,
        _limit: usize,
        verbose: u8,
    ) -> Result<CandidateList> {
        // For now, we only use the first query variant for semantic search
        let query = query_variants
            .first()
            .map(|q| q.text.as_str())
            .unwrap_or("");

        let segments: Vec<_> = corpus
            .documents
            .iter()
            .flat_map(|document| document.segments().iter().cloned())
            .collect();

        if segments.is_empty() {
            return Ok(CandidateList {
                results: Vec::new(),
            });
        }

        // Split segments into those with embeddings and those without
        let (cached, missing): (Vec<_>, Vec<_>) = segments.into_iter().partition(|s| s.embedding.is_some());

        let mut segment_hits = Vec::new();

        if !cached.is_empty() {
            crate::trace!(2, verbose, "    vector: scoring {} segments from cache", cached.len());
            let cached_start = std::time::Instant::now();
            segment_hits.extend(score_segments_manually(&self.dense, query, &cached)?);
            crate::trace!(2, verbose, "    vector: cached scoring complete in {:.2?}", cached_start.elapsed());
        }

        if !missing.is_empty() {
            crate::trace!(2, verbose, "    vector: scoring {} missing segments via inference", missing.len());
            let missing_start = std::time::Instant::now();
            segment_hits.extend(self.dense.score_segments(query, &missing)?);
            crate::trace!(2, verbose, "    vector: inference complete in {:.2?}", missing_start.elapsed());
        }

        let document_hits = aggregate_segment_hits(&segment_hits);

        let results = document_hits
            .into_iter()
            .map(|s| {
                crate::trace!(3, verbose, "      vector score: {:.4} for {}", s.score, s.path.display());
                let mut id = s.id.clone();
                if id.starts_with("./") {
                    id = id.chars().skip(2).collect();
                }
                Candidate {
                    id,
                    path: s.path,
                    score: s.score,
                    contributors: vec![ContributorScore {
                        retriever: RetrieverPolicy::SegmentVector,
                        score: s.score,
                    }],
                    snippet: Some(s.best_segment_text),
                    snippet_location: Some(s.best_segment_label),
                }
            })
            .collect();

        Ok(CandidateList { results })
    }

    fn policy(&self) -> RetrieverPolicy {
        RetrieverPolicy::SegmentVector
    }
}

pub struct PhraseRetriever;

impl Retriever for PhraseRetriever {
    fn retrieve(
        &self,
        query_variants: &[QueryVariant],
        corpus: &PreparedCorpus,
        limit: usize,
        verbose: u8,
    ) -> Result<CandidateList> {
        let query = query_variants
            .first()
            .map(|q| q.text.to_lowercase())
            .unwrap_or_default();

        if query.is_empty() {
            return Ok(CandidateList {
                results: Vec::new(),
            });
        }

        crate::trace!(2, verbose, "    phrase: scanning {} documents", corpus.documents.len());
        let phrase_start = std::time::Instant::now();
        let mut results = Vec::new();
        for document in corpus.documents {
            let text = document.text.to_lowercase();
            if text.contains(&query) {
                crate::trace!(3, verbose, "      phrase match: {}", document.path.display());
                // For phrase search, we just give it a 1.0 score if it contains the phrase
                // This is a very simple implementation
                results.push(Candidate {
                    id: document.id.clone(),
                    path: document.path.clone(),
                    score: 1.0,
                    contributors: vec![ContributorScore {
                        retriever: RetrieverPolicy::Phrase,
                        score: 1.0,
                    }],
                    snippet: None,
                    snippet_location: None,
                });
            }
        }
        crate::trace!(2, verbose, "    phrase: scan complete in {:.2?}", phrase_start.elapsed());

        Ok(CandidateList {
            results: results.into_iter().take(limit).collect(),
        })
    }

    fn policy(&self) -> RetrieverPolicy {
        RetrieverPolicy::Phrase
    }
}

pub struct Bm25Retriever;

impl Retriever for Bm25Retriever {
    fn retrieve(
        &self,
        query_variants: &[QueryVariant],
        corpus: &PreparedCorpus,
        limit: usize,
        verbose: u8,
    ) -> Result<CandidateList> {
        let index = corpus
            .bm25_index
            .ok_or_else(|| anyhow::anyhow!("BM25 retriever requires a BM25 index"))?;

        // For now, we only use the first query variant for BM25
        let query = query_variants
            .first()
            .map(|q| q.text.as_str())
            .unwrap_or("");

        crate::trace!(2, verbose, "    bm25: scoring {} documents", corpus.documents.len());
        let bm25_start = std::time::Instant::now();
        let scored = index.score(query);
        crate::trace!(2, verbose, "    bm25: score complete in {:.2?}", bm25_start.elapsed());
        let results = scored
            .into_iter()
            .take(limit)
            .map(|s| {
                crate::trace!(3, verbose, "      bm25 score: {:.4} for {}", s.score, s.path.display());
                let mut id = s.id.clone();
                if id.starts_with("./") {
                    id = id.chars().skip(2).collect();
                }
                Candidate {
                    id,
                    path: s.path,
                    score: s.score,
                    contributors: vec![ContributorScore {
                        retriever: RetrieverPolicy::Bm25,
                        score: s.score,
                    }],
                    snippet: None, // Snippet resolution happens later
                    snippet_location: None,
                }
            })
            .collect();

        Ok(CandidateList { results })
    }

    fn policy(&self) -> RetrieverPolicy {
        RetrieverPolicy::Bm25
    }
}

pub struct RrfFuser;

impl Fuser for RrfFuser {
    fn fuse(&self, candidate_lists: &[CandidateList], limit: usize, _verbose: u8) -> Result<CandidateList> {
        // Implement RRF fusion
        // This can be adapted from src/hybrid.rs
        // Wait, I need a f64 RRF_K constant.
        const RRF_K: f64 = 60.0;

        let mut documents = std::collections::HashMap::new();

        for list in candidate_lists {
            for (index, candidate) in list.results.iter().enumerate() {
                let mut id = candidate.id.clone();
                if id.starts_with("./") {
                    id = id.chars().skip(2).collect();
                }
                let entry = documents
                    .entry(id.clone())
                    .or_insert_with(|| Candidate {
                        id,
                        path: candidate.path.clone(),
                        score: 0.0,
                        contributors: Vec::new(),
                        snippet: candidate.snippet.clone(),
                        snippet_location: candidate.snippet_location.clone(),
                    });

                entry.score += 1.0 / (RRF_K + (index + 1) as f64);
                entry.contributors.extend(candidate.contributors.clone());
                if entry.snippet.is_none() {
                    entry.snippet = candidate.snippet.clone();
                    entry.snippet_location = candidate.snippet_location.clone();
                }
            }
        }

        let mut results: Vec<_> = documents.into_values().collect();
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });

        Ok(CandidateList {
            results: results.into_iter().take(limit).collect(),
        })
    }
}

pub struct NoExpander;

impl Expander for NoExpander {
    fn expand(&self, query: &str) -> Vec<QueryVariant> {
        vec![QueryVariant {
            text: query.to_string(),
            weight: 1.0,
        }]
    }
}

pub struct SynonymExpander;

impl Expander for SynonymExpander {
    fn expand(&self, query: &str) -> Vec<QueryVariant> {
        // A very simple mock synonym expander
        let mut variants = vec![QueryVariant {
            text: query.to_string(),
            weight: 1.0,
        }];

        if query.contains("search") {
            variants.push(QueryVariant {
                text: query.replace("search", "retrieval"),
                weight: 0.8,
            });
        }

        variants
    }
}

pub struct NoReranker;

impl Reranker for NoReranker {
    fn rerank(
        &self,
        _query: &str,
        candidates: CandidateList,
        _limit: usize,
    ) -> Result<CandidateList> {
        Ok(candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrf_fuser_preserves_provenance() {
        let fuser = RrfFuser;
        let list1 = CandidateList {
            results: vec![Candidate {
                id: "doc1".to_string(),
                path: std::path::PathBuf::from("path1"),
                score: 1.0,
                contributors: vec![ContributorScore {
                    retriever: RetrieverPolicy::Bm25,
                    score: 1.0,
                }],
                snippet: None,
                snippet_location: None,
            }],
        };
        let list2 = CandidateList {
            results: vec![Candidate {
                id: "doc1".to_string(),
                path: std::path::PathBuf::from("path1"),
                score: 0.8,
                contributors: vec![ContributorScore {
                    retriever: RetrieverPolicy::SegmentVector,
                    score: 0.8,
                }],
                snippet: None,
                snippet_location: None,
            }],
        };

        let fused = fuser.fuse(&[list1, list2], 10, 0).unwrap();
        assert_eq!(fused.results.len(), 1);
        assert_eq!(fused.results[0].contributors.len(), 2);
        assert!(
            fused.results[0]
                .contributors
                .iter()
                .any(|c| c.retriever == RetrieverPolicy::Bm25)
        );
        assert!(
            fused.results[0]
                .contributors
                .iter()
                .any(|c| c.retriever == RetrieverPolicy::SegmentVector)
        );
    }
}

pub mod cli;
pub use cli::*;
