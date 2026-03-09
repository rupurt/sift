use super::domain::*;
use crate::dense::DenseReranker;
use crate::vector::retrieve_semantic_documents;
use anyhow::Result;

pub struct SegmentVectorRetriever {
    pub dense: DenseReranker,
}

impl Retriever for SegmentVectorRetriever {
    fn retrieve(
        &self,
        query_variants: &[QueryVariant],
        corpus: &PreparedCorpus,
        _limit: usize,
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

        let semantic =
            retrieve_semantic_documents(&self.dense, query, &segments, corpus.documents.len())?;

        let results = semantic
            .into_iter()
            .map(|s| Candidate {
                id: s.id,
                path: s.path,
                score: s.score,
                contributors: vec![ContributorScore {
                    retriever: RetrieverPolicy::SegmentVector,
                    score: s.score,
                }],
                snippet: Some(s.best_segment_text),
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

        let mut results = Vec::new();
        for document in corpus.documents {
            let text = document.text.to_lowercase();
            if text.contains(&query) {
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
                });
            }
        }

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
    ) -> Result<CandidateList> {
        let index = corpus
            .bm25_index
            .ok_or_else(|| anyhow::anyhow!("BM25 retriever requires a BM25 index"))?;

        // For now, we only use the first query variant for BM25
        let query = query_variants
            .first()
            .map(|q| q.text.as_str())
            .unwrap_or("");

        let scored = index.score(query);
        let results = scored
            .into_iter()
            .take(limit)
            .map(|s| Candidate {
                id: s.id,
                path: s.path,
                score: s.score,
                contributors: vec![ContributorScore {
                    retriever: RetrieverPolicy::Bm25,
                    score: s.score,
                }],
                snippet: None, // Snippet resolution happens later
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
    fn fuse(&self, candidate_lists: &[CandidateList], limit: usize) -> Result<CandidateList> {
        // Implement RRF fusion
        // This can be adapted from src/hybrid.rs
        // Wait, I need a f64 RRF_K constant.
        const RRF_K: f64 = 60.0;

        let mut documents = std::collections::HashMap::new();

        for list in candidate_lists {
            for (index, candidate) in list.results.iter().enumerate() {
                let entry = documents
                    .entry(candidate.id.clone())
                    .or_insert_with(|| Candidate {
                        id: candidate.id.clone(),
                        path: candidate.path.clone(),
                        score: 0.0,
                        contributors: Vec::new(),
                        snippet: candidate.snippet.clone(),
                    });

                entry.score += 1.0 / (RRF_K + (index + 1) as f64);
                entry.contributors.extend(candidate.contributors.clone());
                if entry.snippet.is_none() {
                    entry.snippet = candidate.snippet.clone();
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
            }],
        };

        let fused = fuser.fuse(&[list1, list2], 10).unwrap();
        assert_eq!(fused.results.len(), 1);
        assert_eq!(fused.results[0].contributors.len(), 2);
        assert!(fused.results[0].contributors.iter().any(|c| c.retriever == RetrieverPolicy::Bm25));
        assert!(fused.results[0].contributors.iter().any(|c| c.retriever == RetrieverPolicy::SegmentVector));
    }
}

pub mod cli;
pub use cli::*;
