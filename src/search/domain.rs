use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::dense::DenseModelSpec;
use crate::extract::SourceKind;
use crate::segment::Segment;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    Bm25,
    Hybrid,
}

impl Engine {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bm25 => "bm25",
            Self::Hybrid => "hybrid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub strategy: String,
    pub query: String,
    pub path: PathBuf,
    pub limit: usize,
    pub shortlist: usize,
    pub dense_model: DenseModelSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResponse {
    pub strategy: String,
    pub root: String,
    pub indexed_files: usize,
    pub skipped_files: usize,
    pub results: Vec<SearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchHit {
    pub path: String,
    pub rank: usize,
    pub score: f64,
    pub location: Option<String>,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub struct LoadedCorpus {
    pub documents: Vec<Document>,
    pub total_bytes: u64,
    pub indexed_files: usize,
    pub skipped_files: usize,
}

impl LoadedCorpus {
    pub fn document_by_id(&self, id: &str) -> Option<&Document> {
        self.documents.iter().find(|document| document.id == id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub path: PathBuf,
    pub source_kind: SourceKind,
    pub length: usize,
    pub terms: HashMap<String, usize>,
    pub text: String,
    pub segments: Vec<Segment>,
}

impl Document {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoredDocument {
    pub id: String,
    pub path: PathBuf,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankedDocument {
    pub id: String,
    pub path: PathBuf,
    pub score: f64,
    pub bm25_score: f64,
    pub dense_score: Option<f64>,
    pub snippet: Option<String>,
}

pub struct Bm25Index {
    pub documents: Vec<Document>,
    pub doc_freq: HashMap<String, usize>,
    pub avg_doc_len: f64,
}

impl Bm25Index {
    pub fn build(documents: &[Document]) -> Self {
        let mut doc_freq = HashMap::new();
        let total_len = documents
            .iter()
            .map(|document| document.length)
            .sum::<usize>();

        for document in documents {
            let unique_terms = document.terms.keys().collect::<HashSet<_>>();
            for term in unique_terms {
                *doc_freq.entry(term.clone()).or_insert(0) += 1;
            }
        }

        let avg_doc_len = if documents.is_empty() {
            0.0
        } else {
            total_len as f64 / documents.len() as f64
        };

        Self {
            documents: documents.to_vec(),
            doc_freq,
            avg_doc_len,
        }
    }

    pub fn score(&self, query: &str) -> Vec<ScoredDocument> {
        if self.documents.is_empty() {
            return Vec::new();
        }

        let mut query_terms = tokenize(query);
        query_terms.sort();
        query_terms.dedup();

        let total_docs = self.documents.len() as f64;
        let mut ranked = Vec::with_capacity(self.documents.len());

        for document in &self.documents {
            let mut score = 0.0;

            for term in &query_terms {
                let tf = document.terms.get(term).copied().unwrap_or(0) as f64;
                if tf == 0.0 {
                    continue;
                }

                let idf = {
                    let doc_freq = self.doc_freq.get(term).copied().unwrap_or(0) as f64;
                    ((total_docs - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln()
                };
                let length_ratio = if self.avg_doc_len > 0.0 {
                    document.length as f64 / self.avg_doc_len
                } else {
                    1.0
                };
                let denominator = tf + 1.5 * (1.0 - 0.75 + 0.75 * length_ratio);
                score += idf * (tf * (1.5 + 1.0) / denominator);
            }

            ranked.push(ScoredDocument {
                id: document.id.clone(),
                path: document.path.clone(),
                score,
            });
        }

        ranked.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.id.cmp(&right.id))
        });
        ranked
    }
}

pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if ch.is_alphanumeric() {
            current.extend(ch.to_lowercase());
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

pub fn build_snippet(text: &str, query: &str) -> String {
    let query_terms = tokenize(query);
    if query_terms.is_empty() {
        return build_simple_snippet(text, 160);
    }

    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let lowercase_text = collapsed.to_lowercase();

    // Find the first occurrence of any query term
    let mut first_match_pos = None;
    for term in &query_terms {
        if let Some(pos) = lowercase_text.find(term) {
            match first_match_pos {
                None => first_match_pos = Some(pos),
                Some(current) if pos < current => first_match_pos = Some(pos),
                _ => {}
            }
        }
    }

    let start_pos = first_match_pos.unwrap_or(0);
    // Move back a bit to get some context, but don't split words
    let context_start = if start_pos > 40 {
        // Find a space around start_pos - 40
        collapsed[..start_pos - 40]
            .rfind(' ')
            .map(|p| p + 1)
            .unwrap_or(0)
    } else {
        0
    };

    let limit = 240; // Show more content
    let mut snippet = collapsed
        .chars()
        .skip(context_start)
        .take(limit)
        .collect::<String>();

    if context_start > 0 {
        snippet = format!("...{}", snippet);
    }
    if collapsed.chars().count() > context_start + limit {
        snippet.push_str("...");
    }

    highlight_matches(&snippet, &query_terms)
}

fn build_simple_snippet(text: &str, limit: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return String::new();
    }

    let mut snippet = collapsed.chars().take(limit).collect::<String>();
    if collapsed.chars().count() > limit {
        snippet.push_str("...");
    }
    snippet
}

fn highlight_matches(text: &str, terms: &[String]) -> String {
    if terms.is_empty() {
        return text.to_string();
    }

    let mut highlighted = text.to_string();
    // Use a regex-like replacement or simple string replace for each term.
    // To avoid nested highlighting or partial matches, we sort terms by length descending.
    let mut sorted_terms = terms.to_vec();
    sorted_terms.sort_by_key(|b| std::cmp::Reverse(b.len()));

    for term in sorted_terms {
        // We need case-insensitive replacement but preserving original case.
        // This is a simple but slightly inefficient approach for a CLI.
        let mut result = String::new();
        let mut last_pos = 0;
        let lowercase_h = highlighted.to_lowercase();

        while let Some(pos) = lowercase_h[last_pos..].find(&term) {
            let actual_pos = last_pos + pos;
            result.push_str(&highlighted[last_pos..actual_pos]);
            result.push_str("\x1b[1;33m"); // Bold Yellow
            result.push_str(&highlighted[actual_pos..actual_pos + term.len()]);
            result.push_str("\x1b[0m"); // Reset
            last_pos = actual_pos + term.len();
        }
        result.push_str(&highlighted[last_pos..]);
        highlighted = result;
    }

    highlighted
}

// --- NEW DOMAIN TYPES ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchPlan {
    pub name: String,
    pub query_expansion: QueryExpansionPolicy,
    pub retrievers: Vec<RetrieverPolicy>,
    pub fusion: FusionPolicy,
    pub reranking: RerankingPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryExpansionPolicy {
    None,
    Synonym,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RetrieverPolicy {
    Bm25,
    Phrase,
    SegmentVector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FusionPolicy {
    Rrf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RerankingPolicy {
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub path: PathBuf,
    pub score: f64,
    pub contributors: Vec<ContributorScore>,
    pub snippet: Option<String>,
    pub snippet_location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContributorScore {
    pub retriever: RetrieverPolicy,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateList {
    pub results: Vec<Candidate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryVariant {
    pub text: String,
    pub weight: f64,
}

pub struct StrategyPreset {
    pub name: String,
    pub plan: SearchPlan,
}

pub struct PreparedCorpus<'a> {
    pub documents: &'a [Document],
    pub bm25_index: Option<&'a Bm25Index>,
}

pub trait Retriever {
    fn retrieve(
        &self,
        query_variants: &[QueryVariant],
        corpus: &PreparedCorpus,
        limit: usize,
    ) -> Result<CandidateList>;
    fn policy(&self) -> RetrieverPolicy;
}

pub trait Fuser {
    fn fuse(&self, candidate_lists: &[CandidateList], limit: usize) -> Result<CandidateList>;
}

pub trait Expander {
    fn expand(&self, query: &str) -> Vec<QueryVariant>;
}

pub trait Reranker {
    fn rerank(&self, query: &str, candidates: CandidateList, limit: usize)
    -> Result<CandidateList>;
}
