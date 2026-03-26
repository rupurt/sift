use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use crate::search::domain::*;
use anyhow::Result;

pub mod cli;
pub mod gemma;
pub mod jina;
pub mod llm_utils;
pub mod qwen;

pub use self::cli::*;
pub use self::gemma::*;
pub use self::jina::*;
pub use self::llm_utils::*;
pub use self::qwen::*;

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
        _query: &str,
        mut candidates: CandidateList,
        limit: usize,
    ) -> Result<CandidateList> {
        candidates.results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        candidates.results.truncate(limit);
        Ok(candidates)
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
        candidates: CandidateList,
        _limit: usize,
    ) -> Result<CandidateList> {
        Ok(candidates)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct RerankerAsGenerative(pub Arc<dyn Reranker>);

impl GenerativeModel for RerankerAsGenerative {
    fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        if let Some(generative) = self.0.as_generative() {
            generative.generate(prompt, max_tokens)
        } else {
            anyhow::bail!("Reranker does not support generation")
        }
    }

    fn start_conversation(&self) -> Result<Box<dyn Conversation>> {
        if let Some(generative) = self.0.as_generative() {
            generative.start_conversation()
        } else {
            anyhow::bail!("Reranker does not support stateful conversations")
        }
    }
}

pub struct RrfFuser;
impl Fuser for RrfFuser {
    fn fuse(&self, lists: &[CandidateList], limit: usize, _verbose: u8) -> Result<CandidateList> {
        let mut scores = HashMap::new();
        let mut results = Vec::new();
        for list in lists {
            for (rank, candidate) in list.results.iter().enumerate() {
                let score = 1.0 / (60.0 + rank as f64 + 1.0);
                *scores.entry(candidate.id.clone()).or_insert(0.0) += score;
                if results
                    .iter()
                    .find(|c: &&Candidate| c.id == candidate.id)
                    .is_none()
                {
                    results.push(candidate.clone());
                }
            }
        }
        for candidate in &mut results {
            candidate.score = *scores.get(&candidate.id).unwrap();
        }
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        results.truncate(limit);
        Ok(CandidateList { results })
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
        vec![QueryVariant {
            text: query.to_string(),
            weight: 1.0,
        }]
    }
}

pub struct LlmExpander {
    pub strategy: Box<dyn GenerativeExpansionStrategy>,
    pub llm: Option<Arc<dyn GenerativeModel>>,
}
impl LlmExpander {
    pub fn new(strategy: Box<dyn GenerativeExpansionStrategy>) -> Self {
        Self {
            strategy,
            llm: None,
        }
    }
    pub fn with_llm(mut self, llm: Arc<dyn GenerativeModel>) -> Self {
        self.llm = Some(llm);
        self
    }
}
impl Expander for LlmExpander {
    fn expand(&self, query: &str) -> Vec<QueryVariant> {
        if let Some(llm) = &self.llm {
            let output = llm.generate(query, 512).unwrap_or_default();
            self.strategy.process_output(query, &output)
        } else {
            vec![QueryVariant {
                text: query.to_string(),
                weight: 1.0,
            }]
        }
    }
}

pub trait GenerativeExpansionStrategy: Send + Sync {
    fn process_output(&self, query: &str, output: &str) -> Vec<QueryVariant>;
}

pub struct HydeStrategy {
    pub custom_prompt: Option<String>,
}
impl GenerativeExpansionStrategy for HydeStrategy {
    fn process_output(&self, _query: &str, output: &str) -> Vec<QueryVariant> {
        vec![QueryVariant {
            text: output.to_string(),
            weight: 1.0,
        }]
    }
}

pub struct SpladeStrategy {
    pub custom_prompt: Option<String>,
}
impl GenerativeExpansionStrategy for SpladeStrategy {
    fn process_output(&self, _query: &str, output: &str) -> Vec<QueryVariant> {
        vec![QueryVariant {
            text: output.to_string(),
            weight: 1.0,
        }]
    }
}

pub struct ClassifiedStrategy {
    pub custom_prompt: Option<String>,
}
impl GenerativeExpansionStrategy for ClassifiedStrategy {
    fn process_output(&self, _query: &str, output: &str) -> Vec<QueryVariant> {
        vec![QueryVariant {
            text: output.to_string(),
            weight: 1.0,
        }]
    }
}

pub struct Bm25Retriever;
impl Retriever for Bm25Retriever {
    fn retrieve(
        &self,
        query: &[QueryVariant],
        corpus: &PreparedCorpus,
        limit: usize,
        _verbose: u8,
    ) -> Result<CandidateList> {
        if let Some(index) = corpus.bm25_index {
            let query_texts: Vec<_> = query.iter().map(|v| v.text.clone()).collect();
            let mut all_query_terms = Vec::new();
            for text in query_texts {
                all_query_terms.extend(tokenize(&text));
            }
            let scores = index.score(&all_query_terms);
            let mut results: Vec<_> = scores
                .into_iter()
                .map(|(doc_id, score)| {
                    let doc = corpus.documents.iter().find(|d| d.id == doc_id).unwrap();
                    Candidate {
                        id: doc_id,
                        path: doc.path.clone(),
                        score,
                        contributors: vec![ContributorScore {
                            retriever: RetrieverPolicy::Bm25,
                            score,
                        }],
                        snippet: None,
                        snippet_location: None,
                    }
                })
                .collect();
            results.truncate(limit);
            Ok(CandidateList { results })
        } else {
            Ok(CandidateList { results: vec![] })
        }
    }
    fn policy(&self) -> RetrieverPolicy {
        RetrieverPolicy::Bm25
    }
}

pub struct PhraseRetriever;
impl Retriever for PhraseRetriever {
    fn retrieve(
        &self,
        _query: &[QueryVariant],
        _corpus: &PreparedCorpus,
        _limit: usize,
        _verbose: u8,
    ) -> Result<CandidateList> {
        Ok(CandidateList { results: vec![] })
    }
    fn policy(&self) -> RetrieverPolicy {
        RetrieverPolicy::Phrase
    }
}

pub struct SegmentVectorRetriever {
    pub embedder: Arc<dyn Embedder>,
}
impl Retriever for SegmentVectorRetriever {
    fn retrieve(
        &self,
        _query: &[QueryVariant],
        _corpus: &PreparedCorpus,
        _limit: usize,
        _verbose: u8,
    ) -> Result<CandidateList> {
        Ok(CandidateList { results: vec![] })
    }
    fn policy(&self) -> RetrieverPolicy {
        RetrieverPolicy::Vector
    }
}
