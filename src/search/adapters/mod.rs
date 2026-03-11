use super::domain::*;
use anyhow::Result;

use crate::vector::{aggregate_segment_hits, score_segments_manually};

pub struct SegmentVectorRetriever {
    pub embedder: std::sync::Arc<dyn Embedder>,
}

impl Retriever for SegmentVectorRetriever {
    fn retrieve(
        &self,
        query_variants: &[QueryVariant],
        corpus: &PreparedCorpus,
        _limit: usize,
        _verbose: u8,
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

        let mut segment_hits = Vec::new();

        tracing::info!("vector: scoring {} segments", segments.len());
        let start = std::time::Instant::now();
        segment_hits.extend(score_segments_manually(
            self.embedder.as_ref(),
            query,
            &segments,
        )?);
        tracing::debug!("vector: scoring complete in {:.2?}", start.elapsed());

        tracing::debug!("vector: aggregating hits");
        let document_hits = aggregate_segment_hits(&segment_hits);
        tracing::debug!("vector: aggregation complete");

        let results = document_hits
            .into_iter()
            .map(|s| {
                tracing::debug!("vector score: {:.4} for {}", s.score, s.path.display());
                let mut id = s.id.clone();
                if id.starts_with("./") {
                    id = id.chars().skip(2).collect();
                }
                Candidate {
                    id,
                    path: s.path,
                    score: s.score,
                    contributors: vec![ContributorScore {
                        retriever: RetrieverPolicy::Vector,
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
        RetrieverPolicy::Vector
    }
}

pub struct PhraseRetriever;

impl Retriever for PhraseRetriever {
    fn retrieve(
        &self,
        query_variants: &[QueryVariant],
        corpus: &PreparedCorpus,
        limit: usize,
        _verbose: u8,
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

        tracing::info!("phrase: scanning {} documents", corpus.documents.len());
        let phrase_start = std::time::Instant::now();
        let mut results = Vec::new();
        for document in corpus.documents {
            let text = document.text.to_lowercase();
            if text.contains(&query) {
                tracing::debug!("phrase match: {}", document.path.display());
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
        tracing::debug!("phrase: scan complete in {:.2?}", phrase_start.elapsed());

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
        _verbose: u8,
    ) -> Result<CandidateList> {
        let index = corpus
            .bm25_index
            .ok_or_else(|| anyhow::anyhow!("BM25 retriever requires a BM25 index"))?;

        // For now, we only use the first query variant for BM25
        let query = query_variants
            .first()
            .map(|q| q.text.as_str())
            .unwrap_or("");

        tracing::info!("bm25: scoring {} documents", corpus.documents.len());
        let bm25_start = std::time::Instant::now();
        let scored = index.score(query);
        tracing::debug!("bm25: score complete in {:.2?}", bm25_start.elapsed());
        let results = scored
            .into_iter()
            .take(limit)
            .map(|s| {
                tracing::debug!("bm25 score: {:.4} for {}", s.score, s.path.display());
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
    fn fuse(
        &self,
        candidate_lists: &[CandidateList],
        limit: usize,
        _verbose: u8,
    ) -> Result<CandidateList> {
        tracing::debug!("fusing {} candidate lists", candidate_lists.len());
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
                let entry = documents.entry(id.clone()).or_insert_with(|| Candidate {
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

        tracing::debug!(
            "fusion complete, sorted {} unique candidates",
            results.len()
        );

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

pub struct LlmExpander {
    pub llm: std::sync::Arc<std::sync::Mutex<Option<std::sync::Arc<dyn GenerativeModel>>>>,
    pub strategy: Box<dyn GenerativeExpansionStrategy>,
}

impl LlmExpander {
    pub fn new(strategy: Box<dyn GenerativeExpansionStrategy>) -> Self {
        Self {
            llm: std::sync::Arc::new(std::sync::Mutex::new(None)),
            strategy,
        }
    }

    pub fn with_llm(self, llm: std::sync::Arc<dyn GenerativeModel>) -> Self {
        *self.llm.lock().unwrap() = Some(llm);
        self
    }
}

impl Expander for LlmExpander {
    fn expand(&self, query: &str) -> Vec<QueryVariant> {
        let llm = self.llm.lock().unwrap();
        if let Some(qwen) = llm.as_ref() {
            let prompt = self.strategy.prompt(query);
            tracing::info!(
                "{}: generating expansion for '{}'...",
                self.strategy.name(),
                query
            );
            match qwen.generate(&prompt, self.strategy.max_tokens()) {
                Ok(generated) => {
                    tracing::info!(
                        "{} generated: '{}'",
                        self.strategy.name(),
                        generated.replace('\n', " ")
                    );
                    return self.strategy.process_output(query, &generated);
                }
                Err(e) => {
                    tracing::error!("{} generation failed: {}", self.strategy.name(), e);
                }
            }
        }

        tracing::info!("{}: using fallback for '{}'", self.strategy.name(), query);
        if self.strategy.name() == "HyDE" {
            vec![
                QueryVariant {
                    text: query.to_string(),
                    weight: 1.0,
                },
                QueryVariant {
                    text: format!("{} hypothetical content", query),
                    weight: 0.8,
                },
            ]
        } else {
            vec![QueryVariant {
                text: query.to_string(),
                weight: 1.0,
            }]
        }
    }
}

pub trait GenerativeExpansionStrategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn prompt(&self, query: &str) -> String;
    fn process_output(&self, query: &str, output: &str) -> Vec<QueryVariant>;
    fn max_tokens(&self) -> usize {
        48
    }
}

pub struct HydeStrategy;
impl GenerativeExpansionStrategy for HydeStrategy {
    fn name(&self) -> &'static str {
        "HyDE"
    }
    fn max_tokens(&self) -> usize {
        128
    }
    fn prompt(&self, query: &str) -> String {
        format!(
            "<|im_start|>system\nYou are a technical assistant. For the given query, generate a concise, hypothetical technical document snippet that would satisfy the user's intent. Focus on code structures, API names, and implementation logic.<|im_end|>\n<|im_start|>user\nQuery: {}\n<|im_end|>\n<|im_start|>assistant\n",
            query
        )
    }
    fn process_output(&self, query: &str, output: &str) -> Vec<QueryVariant> {
        vec![
            QueryVariant {
                text: query.to_string(),
                weight: 1.0,
            },
            QueryVariant {
                text: output.to_string(),
                weight: 0.8,
            },
        ]
    }
}

pub struct SpladeStrategy;
impl GenerativeExpansionStrategy for SpladeStrategy {
    fn name(&self) -> &'static str {
        "SPLADE"
    }
    fn prompt(&self, query: &str) -> String {
        format!(
            "<|im_start|>system\nYou are a technical search expert. For the given query, provide 5-8 technical terms that are semantically related but NOT necessarily present in the query. Return ONLY the keywords separated by commas.<|im_end|>\n<|im_start|>user\nQuery: {}\n<|im_end|>\n<|im_start|>assistant\n",
            query
        )
    }
    fn process_output(&self, query: &str, output: &str) -> Vec<QueryVariant> {
        let mut variants = vec![QueryVariant {
            text: query.to_string(),
            weight: 1.0,
        }];
        for term in output.split(',') {
            let term = term.trim();
            if !term.is_empty() {
                variants.push(QueryVariant {
                    text: term.to_string(),
                    weight: 0.4,
                });
            }
        }
        variants
    }
}

pub struct ClassifiedStrategy;
impl GenerativeExpansionStrategy for ClassifiedStrategy {
    fn name(&self) -> &'static str {
        "CLASSIFIED"
    }
    fn prompt(&self, query: &str) -> String {
        format!(
            "<|im_start|>system\nYou are a technical classifier. Identify the technical intent of the query (e.g., NAVIGATION, LOGIC, DOCS, BUGFIX, TEST). Return the classification and 3 highly specific keywords for that intent. Format: CATEGORY: keyword1, keyword2, keyword3<|im_end|>\n<|im_start|>user\nQuery: {}\n<|im_end|>\n<|im_start|>assistant\n",
            query
        )
    }
    fn process_output(&self, query: &str, output: &str) -> Vec<QueryVariant> {
        let mut variants = vec![QueryVariant {
            text: query.to_string(),
            weight: 1.0,
        }];
        if let Some((_category, keywords)) = output.split_once(':') {
            for term in keywords.split(',') {
                let term = term.trim();
                if !term.is_empty() {
                    variants.push(QueryVariant {
                        text: term.to_string(),
                        weight: 0.6,
                    });
                }
            }
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct PositionAwareReranker;

impl Reranker for PositionAwareReranker {
    fn rerank(
        &self,
        query: &str,
        mut candidates: CandidateList,
        limit: usize,
    ) -> Result<CandidateList> {
        let query_terms = super::domain::tokenize(query);
        if query_terms.is_empty() {
            return Ok(candidates);
        }

        for candidate in &mut candidates.results {
            let mut bonus = 0.0;

            // 1. Filename Bonus
            let filename = candidate
                .path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("")
                .to_lowercase();

            for term in &query_terms {
                if filename.contains(term) {
                    bonus += 0.05;
                    break; // Only one filename bonus
                }
            }

            // 2. Heading (Location) Bonus
            if let Some(location) = &candidate.snippet_location {
                let location_lower = location.to_lowercase();
                for term in &query_terms {
                    if location_lower.contains(term) {
                        bonus += 0.02;
                        break; // Only one location bonus
                    }
                }
            }

            candidate.score += bonus;
        }

        // Re-sort after applying bonuses
        candidates.results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });

        Ok(CandidateList {
            results: candidates.results.into_iter().take(limit).collect(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct MockLlmReranker;

impl Reranker for MockLlmReranker {
    fn rerank(
        &self,
        _query: &str,
        mut candidates: CandidateList,
        limit: usize,
    ) -> Result<CandidateList> {
        // A mock LLM reranker that slightly adjusts top scores to simulate "intelligent" re-scoring.
        // It just adds a small random-ish bonus to the first few results.
        for (i, candidate) in candidates.results.iter_mut().enumerate().take(3) {
            candidate.score += 0.01 / (i + 1) as f64;
        }

        candidates.results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });

        Ok(CandidateList {
            results: candidates.results.into_iter().take(limit).collect(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
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
                    retriever: RetrieverPolicy::Vector,
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
                .any(|c| c.retriever == RetrieverPolicy::Vector)
        );
    }
}

pub struct RerankerAsGenerative(pub std::sync::Arc<dyn Reranker>);

impl GenerativeModel for RerankerAsGenerative {
    fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        if let Some(generative) = self.0.as_generative() {
            generative.generate(prompt, max_tokens)
        } else {
            anyhow::bail!("Reranker does not support generation")
        }
    }
}
pub mod cli;
pub mod jina;
pub mod llm_utils;
pub mod qwen;
#[allow(unused_imports)]
pub use cli::*;
