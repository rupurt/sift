use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use crate::search::domain::*;
use crate::vector::{aggregate_segment_hits, score_segments_manually};
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
            let prompt = self.strategy.prompt(query);
            match llm.generate(&prompt, self.strategy.max_tokens()) {
                Ok(output) => self.strategy.process_output(query, &output),
                Err(err) => {
                    tracing::warn!("query expansion failed, falling back to the raw query: {err}");
                    vec![original_query_variant(query)]
                }
            }
        } else {
            vec![original_query_variant(query)]
        }
    }
}

pub trait GenerativeExpansionStrategy: Send + Sync {
    fn prompt(&self, query: &str) -> String;
    fn process_output(&self, query: &str, output: &str) -> Vec<QueryVariant>;
    fn max_tokens(&self) -> usize {
        128
    }
}

pub struct HydeStrategy {
    pub custom_prompt: Option<String>,
}
impl GenerativeExpansionStrategy for HydeStrategy {
    fn prompt(&self, query: &str) -> String {
        render_expansion_prompt(
            self.custom_prompt.as_deref(),
            "Write a short hypothetical passage that would likely appear in a relevant document. Preserve exact scientific terms from the query. Return only the passage.",
            query,
        )
    }

    fn process_output(&self, query: &str, output: &str) -> Vec<QueryVariant> {
        expansion_variants(query, output, 96, 0.7)
    }
}

pub struct SpladeStrategy {
    pub custom_prompt: Option<String>,
}
impl GenerativeExpansionStrategy for SpladeStrategy {
    fn prompt(&self, query: &str) -> String {
        render_expansion_prompt(
            self.custom_prompt.as_deref(),
            "Generate a concise list of retrieval keywords and short phrases for the query. Keep domain terms, names, and negations. Return only the terms.",
            query,
        )
    }

    fn process_output(&self, query: &str, output: &str) -> Vec<QueryVariant> {
        expansion_variants(query, output, 32, 0.5)
    }
}

pub struct ClassifiedStrategy {
    pub custom_prompt: Option<String>,
}
impl GenerativeExpansionStrategy for ClassifiedStrategy {
    fn prompt(&self, query: &str) -> String {
        render_expansion_prompt(
            self.custom_prompt.as_deref(),
            "Generate a concise retrieval expansion for the query that highlights claim type, entities, and discriminative scientific terms. Return only the expansion text.",
            query,
        )
    }

    fn process_output(&self, query: &str, output: &str) -> Vec<QueryVariant> {
        expansion_variants(query, output, 32, 0.5)
    }
}

fn render_expansion_prompt(
    custom_prompt: Option<&str>,
    default_instruction: &str,
    query: &str,
) -> String {
    match custom_prompt {
        Some(template) if template.contains("{query}") => template.replace("{query}", query),
        Some(template) => format!(
            "<|im_start|>system\n{}\n<|im_end|>\n<|im_start|>user\n{}\n<|im_end|>\n<|im_start|>assistant\n",
            template.trim(),
            query.trim()
        ),
        None => format!(
            "<|im_start|>system\n{}\n<|im_end|>\n<|im_start|>user\n{}\n<|im_end|>\n<|im_start|>assistant\n",
            default_instruction.trim(),
            query.trim()
        ),
    }
}

fn original_query_variant(query: &str) -> QueryVariant {
    QueryVariant {
        text: query.trim().to_string(),
        weight: 1.0,
    }
}

fn expansion_variants(
    query: &str,
    output: &str,
    max_terms: usize,
    expansion_weight: f64,
) -> Vec<QueryVariant> {
    let normalized = output
        .replace("<|im_end|>", " ")
        .replace("<|endoftext|>", " ")
        .split_whitespace()
        .take(max_terms)
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .trim_matches('"')
        .trim()
        .to_string();

    let mut variants = vec![original_query_variant(query)];

    if !normalized.is_empty() && normalized != query.trim() {
        variants.push(QueryVariant {
            text: normalized,
            weight: expansion_weight,
        });
    }

    variants
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
            let term_weights = weighted_query_terms(query);
            let scores = score_bm25_terms(index, &term_weights);
            let mut results: Vec<_> = scores
                .into_iter()
                .map(|(artifact_id, score)| {
                    let doc = corpus
                        .artifacts
                        .iter()
                        .find(|d| d.id == artifact_id)
                        .unwrap();
                    Candidate {
                        id: artifact_id,
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
        query: &[QueryVariant],
        corpus: &PreparedCorpus,
        limit: usize,
        _verbose: u8,
    ) -> Result<CandidateList> {
        let mut results = Vec::new();

        for document in corpus.artifacts {
            let mut best_score = 0.0;
            let mut best_snippet = None;
            let mut best_location = None;

            for segment in document.segments() {
                let score = score_segment_phrases(query, &segment.text);
                if score > best_score {
                    best_score = score;
                    best_snippet = Some(segment.text.clone());
                    best_location = Some(segment.label.clone());
                }
            }

            if best_score > 0.0 {
                results.push(Candidate {
                    id: document.id.clone(),
                    path: document.path.clone(),
                    score: best_score,
                    contributors: vec![ContributorScore {
                        retriever: RetrieverPolicy::Phrase,
                        score: best_score,
                    }],
                    snippet: best_snippet,
                    snippet_location: best_location,
                });
            }
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
        query: &[QueryVariant],
        corpus: &PreparedCorpus,
        limit: usize,
        _verbose: u8,
    ) -> Result<CandidateList> {
        if query.is_empty() || corpus.artifacts.is_empty() {
            return Ok(CandidateList { results: vec![] });
        }

        let segments = corpus
            .artifacts
            .iter()
            .flat_map(|document| document.segments.iter().cloned())
            .collect::<Vec<_>>();
        if segments.is_empty() {
            return Ok(CandidateList { results: vec![] });
        }

        let mut combined_hits = Vec::new();
        for variant in query {
            let mut hits =
                score_segments_manually(self.embedder.as_ref(), &variant.text, &segments)?;
            for hit in &mut hits {
                hit.score *= variant.weight;
            }
            combined_hits.extend(
                hits.into_iter()
                    .filter(|hit| hit.score.is_finite() && hit.score > 0.0),
            );
        }

        let results = aggregate_segment_hits(&combined_hits)
            .into_iter()
            .take(limit.max(1))
            .map(|document| Candidate {
                id: document.id,
                path: document.path,
                score: document.score,
                contributors: vec![ContributorScore {
                    retriever: RetrieverPolicy::Vector,
                    score: document.score,
                }],
                snippet: Some(document.best_segment_text),
                snippet_location: Some(document.best_segment_label),
            })
            .collect();

        Ok(CandidateList { results })
    }
    fn policy(&self) -> RetrieverPolicy {
        RetrieverPolicy::Vector
    }
}

fn weighted_query_terms(query: &[QueryVariant]) -> HashMap<String, f64> {
    let mut term_weights = HashMap::new();

    for variant in query {
        if variant.weight <= 0.0 {
            continue;
        }
        for term in tokenize(&variant.text) {
            *term_weights.entry(term).or_insert(0.0) += variant.weight;
        }
    }

    term_weights
}

fn score_bm25_terms(index: &Bm25Index, term_weights: &HashMap<String, f64>) -> Vec<(String, f64)> {
    let mut scores = HashMap::new();
    let k1 = 1.2;
    let b = 0.75;

    for (term, query_weight) in term_weights {
        if *query_weight <= 0.0 {
            continue;
        }
        let Some(&df) = index.doc_freq.get(term) else {
            continue;
        };
        let idf = ((index.num_docs as f64 - df as f64 + 0.5) / (df as f64 + 0.5) + 1.0).ln();

        for (artifact_id, terms) in &index.term_freqs {
            let Some(&tf) = terms.get(term) else {
                continue;
            };
            let doc_len = *index.doc_lengths.get(artifact_id).unwrap_or(&0) as f64;
            let tf = tf as f64;
            let base_score =
                idf * (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * doc_len / index.avg_doc_len));
            *scores.entry(artifact_id.clone()).or_insert(0.0) += base_score * query_weight;
        }
    }

    let mut results = scores.into_iter().collect::<Vec<_>>();
    results.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    results
}

fn score_segment_phrases(query: &[QueryVariant], segment_text: &str) -> f64 {
    let haystack = segment_text.to_lowercase();
    let mut score = 0.0;

    for variant in query {
        if variant.weight <= 0.0 {
            continue;
        }
        for phrase in phrase_candidates(&variant.text) {
            if haystack.contains(&phrase) {
                score += phrase.split_whitespace().count() as f64 * variant.weight;
            }
        }
    }

    score
}

fn phrase_candidates(text: &str) -> Vec<String> {
    let tokens = tokenize(text);
    if tokens.len() < 2 {
        return Vec::new();
    }

    let mut phrases = Vec::new();
    let max_window = tokens.len().min(4);
    for window in (2..=max_window).rev() {
        for chunk in tokens.windows(window) {
            phrases.push(chunk.join(" "));
        }
    }
    phrases.sort();
    phrases.dedup();
    phrases
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;
    use crate::extract::SourceKind;
    use crate::segment::Segment;

    struct EmptyConversation;

    impl Conversation for EmptyConversation {
        fn send(&mut self, _message: &str, _max_tokens: usize) -> Result<String> {
            Ok(String::new())
        }

        fn history(&self) -> &[String] {
            &[]
        }
    }

    struct StaticGenerativeModel {
        output: String,
    }

    impl GenerativeModel for StaticGenerativeModel {
        fn generate(&self, _prompt: &str, _max_tokens: usize) -> Result<String> {
            Ok(self.output.clone())
        }

        fn start_conversation(&self) -> Result<Box<dyn Conversation>> {
            Ok(Box::new(EmptyConversation))
        }
    }

    struct StaticEmbedder;

    impl Embedder for StaticEmbedder {
        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            Ok(texts
                .iter()
                .map(|text| {
                    if text.contains("alpha") {
                        vec![1.0, 0.0]
                    } else if text.contains("beta") {
                        vec![0.0, 1.0]
                    } else {
                        vec![0.0, 0.0]
                    }
                })
                .collect())
        }

        fn dimension(&self) -> usize {
            2
        }
    }

    #[test]
    fn llm_expander_keeps_the_original_query_when_generation_is_empty() {
        let expander = LlmExpander::new(Box::new(SpladeStrategy {
            custom_prompt: None,
        }))
        .with_llm(Arc::new(StaticGenerativeModel {
            output: String::new(),
        }));

        let variants = expander.expand("alpha query");

        assert_eq!(variants.len(), 1);
        assert_eq!(variants[0].text, "alpha query");
    }

    #[test]
    fn llm_expander_keeps_the_original_query_when_generation_is_non_empty() {
        let expander = LlmExpander::new(Box::new(SpladeStrategy {
            custom_prompt: None,
        }))
        .with_llm(Arc::new(StaticGenerativeModel {
            output: "alpha biomaterial keywords".to_string(),
        }));

        let variants = expander.expand("alpha query");

        assert_eq!(variants[0].text, "alpha query");
        assert_eq!(variants[1].text, "alpha biomaterial keywords");
    }

    #[test]
    fn vector_retriever_scores_segment_embeddings() {
        let retriever = SegmentVectorRetriever {
            embedder: Arc::new(StaticEmbedder),
        };
        let corpus = sample_corpus();
        let query = vec![QueryVariant {
            text: "alpha query".to_string(),
            weight: 1.0,
        }];

        let results = retriever
            .retrieve(&query, &corpus, 5, 0)
            .expect("vector retrieval");

        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].id, "doc-alpha");
        assert_eq!(results.results[0].snippet.as_deref(), Some("alpha body"));
    }

    #[test]
    fn bm25_retriever_respects_variant_weights() {
        let artifacts = sample_corpus().artifacts.to_vec();
        let index = Bm25Index::build(&artifacts);
        let prepared = PreparedCorpus {
            artifacts: Box::leak(artifacts.into_boxed_slice()),
            bm25_index: Some(Box::leak(Box::new(index))),
        };
        let retriever = Bm25Retriever;

        let results = retriever
            .retrieve(
                &[
                    QueryVariant {
                        text: "alpha".to_string(),
                        weight: 1.0,
                    },
                    QueryVariant {
                        text: "beta".to_string(),
                        weight: 0.0,
                    },
                ],
                &prepared,
                5,
                0,
            )
            .expect("bm25 retrieval");

        assert_eq!(results.results[0].id, "doc-alpha");
    }

    fn sample_corpus() -> PreparedCorpus<'static> {
        let artifacts = vec![
            ContextArtifact {
                id: "doc-alpha".to_string(),
                kind: ContextArtifactKind::File,
                path: "alpha.txt".into(),
                source_kind: SourceKind::Text,
                length: 2,
                terms: document_terms("alpha body"),
                text: "alpha body".to_string(),
                segments: vec![Segment {
                    id: "doc-alpha::segment:0001".to_string(),
                    artifact_id: "doc-alpha".to_string(),
                    path: "alpha.txt".into(),
                    source_kind: SourceKind::Text,
                    ordinal: 1,
                    label: "section 1".to_string(),
                    text: "alpha body".to_string(),
                    embedding: Some(vec![1.0, 0.0]),
                }],
                provenance: ArtifactProvenance {
                    adapter: AcquisitionAdapterKind::FileSystem,
                    source: "alpha.txt".to_string(),
                    synthetic: false,
                },
                freshness: ArtifactFreshness {
                    observed_unix_secs: 0,
                    modified_unix_secs: None,
                },
                budget: ArtifactBudget::from_text("alpha body", 1),
            },
            ContextArtifact {
                id: "doc-beta".to_string(),
                kind: ContextArtifactKind::File,
                path: "beta.txt".into(),
                source_kind: SourceKind::Text,
                length: 2,
                terms: document_terms("beta body"),
                text: "beta body".to_string(),
                segments: vec![Segment {
                    id: "doc-beta::segment:0001".to_string(),
                    artifact_id: "doc-beta".to_string(),
                    path: "beta.txt".into(),
                    source_kind: SourceKind::Text,
                    ordinal: 1,
                    label: "section 1".to_string(),
                    text: "beta body".to_string(),
                    embedding: Some(vec![0.0, 1.0]),
                }],
                provenance: ArtifactProvenance {
                    adapter: AcquisitionAdapterKind::FileSystem,
                    source: "beta.txt".to_string(),
                    synthetic: false,
                },
                freshness: ArtifactFreshness {
                    observed_unix_secs: 0,
                    modified_unix_secs: None,
                },
                budget: ArtifactBudget::from_text("beta body", 1),
            },
        ];
        let leaked = Box::leak(artifacts.into_boxed_slice());
        PreparedCorpus {
            artifacts: leaked,
            bm25_index: None,
        }
    }

    fn document_terms(text: &str) -> HashMap<String, usize> {
        crate::search::tokenize(text)
            .into_iter()
            .fold(HashMap::new(), |mut terms, term| {
                *terms.entry(term).or_insert(0) += 1;
                terms
            })
    }
}
