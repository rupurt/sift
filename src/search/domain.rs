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

use crate::system::Telemetry;
use std::sync::{Arc, RwLock};

pub type QueryEmbeddingCache = Arc<RwLock<HashMap<String, Vec<f32>>>>;

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub strategy: String,
    pub query: String,
    pub path: PathBuf,
    pub limit: usize,
    pub shortlist: usize,
    pub dense_model: DenseModelSpec,
    pub verbose: u8,
    pub retrievers: Option<Vec<RetrieverPolicy>>,
    pub fusion: Option<FusionPolicy>,
    pub reranking: Option<RerankingPolicy>,
    pub telemetry: Arc<Telemetry>,
    pub cache_dir: Option<PathBuf>,
    pub query_cache: Option<QueryEmbeddingCache>,
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
pub struct SearchTelemetry {
    pub heuristic_hit_rate: f64,
    pub blob_hit_rate: f64,
    pub embedding_hit_rate: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScoreConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchHit {
    pub path: String,
    pub rank: usize,
    pub score: f64,
    pub confidence: ScoreConfidence,
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

            if score > 0.0 {
                ranked.push(ScoredDocument {
                    id: document.id.clone(),
                    path: document.path.clone(),
                    score,
                });
            }
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

// --- NEW DOMAIN TYPES ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchPlan {
    pub name: String,
    pub query_expansion: QueryExpansionPolicy,
    pub retrievers: Vec<RetrieverPolicy>,
    pub fusion: FusionPolicy,
    pub reranking: RerankingPolicy,
}

impl SearchPlan {
    pub fn categorize_score(&self, score: f64) -> ScoreConfidence {
        // RRF_K is constant at 60.0 currently.
        // Max score per retriever is 1 / (60 + 1) ~= 0.01639.
        let max_possible = self.retrievers.len() as f64 / 61.0;
        if max_possible == 0.0 {
            return ScoreConfidence::Low;
        }

        let normalized = score / max_possible;

        if normalized > 0.7 {
            ScoreConfidence::High
        } else if normalized > 0.3 {
            ScoreConfidence::Medium
        } else {
            ScoreConfidence::Low
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryExpansionPolicy {
    None,
    Synonym,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum RetrieverPolicy {
    Bm25,
    Phrase,
    Vector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum FusionPolicy {
    Rrf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum RerankingPolicy {
    None,
    PositionAware,
    Llm,
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

pub trait CorpusRepository: Send + Sync {
    fn load(
        &self,
        path: &std::path::Path,
        ignore: Option<&crate::config::Ignore>,
        verbose: u8,
        embedder: Option<&dyn Embedder>,
        telemetry: &crate::system::Telemetry,
        cache_base: Option<&std::path::Path>,
    ) -> Result<LoadedCorpus>;
}

pub trait Embedder: Send + Sync {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
}

pub struct CachedEmbedder {
    pub inner: Arc<dyn Embedder>,
    pub cache: QueryEmbeddingCache,
}

impl Embedder for CachedEmbedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        let mut missing_indices = Vec::new();
        let mut missing_texts = Vec::new();

        {
            let cache = self.cache.read().unwrap();
            for (i, text) in texts.iter().enumerate() {
                if let Some(embedding) = cache.get(text) {
                    tracing::debug!("query cache hit for: '{}'", text);
                    results.push((i, embedding.clone()));
                } else {
                    tracing::debug!("query cache miss for: '{}'", text);
                    missing_indices.push(i);
                    missing_texts.push(text.clone());
                }
            }
        }

        if !missing_texts.is_empty() {
            let embeddings = self.inner.embed(&missing_texts)?;
            let mut cache = self.cache.write().unwrap();
            for (i, embedding) in missing_indices.into_iter().zip(embeddings) {
                cache.insert(texts[i].clone(), embedding.clone());
                results.push((i, embedding));
            }
        }

        results.sort_by_key(|(i, _)| *i);
        Ok(results.into_iter().map(|(_, e)| e).collect())
    }

    fn dimension(&self) -> usize {
        self.inner.dimension()
    }
}

pub trait Retriever: Send + Sync {
    fn retrieve(
        &self,
        query_variants: &[QueryVariant],
        corpus: &PreparedCorpus,
        limit: usize,
        verbose: u8,
    ) -> Result<CandidateList>;
    fn policy(&self) -> RetrieverPolicy;
}

pub trait Fuser: Send + Sync {
    fn fuse(
        &self,
        candidate_lists: &[CandidateList],
        limit: usize,
        verbose: u8,
    ) -> Result<CandidateList>;
}

pub trait Expander: Send + Sync {
    fn expand(&self, query: &str) -> Vec<QueryVariant>;
}

pub trait Reranker: Send + Sync {
    fn rerank(&self, query: &str, candidates: CandidateList, limit: usize)
    -> Result<CandidateList>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockEmbedder {
        call_count: Arc<AtomicUsize>,
    }

    impl Embedder for MockEmbedder {
        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(texts.iter().map(|_| vec![0.0; 384]).collect())
        }

        fn dimension(&self) -> usize {
            384
        }
    }

    #[test]
    fn cached_embedder_avoids_redundant_calls() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let inner = Arc::new(MockEmbedder {
            call_count: call_count.clone(),
        });
        let cache = Arc::new(RwLock::new(HashMap::new()));
        let embedder = CachedEmbedder {
            inner: inner.clone(),
            cache: cache.clone(),
        };

        let texts = vec!["query1".to_string(), "query2".to_string()];

        // First call: 1 call to inner (for both texts)
        let res1 = embedder.embed(&texts).unwrap();
        assert_eq!(res1.len(), 2);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Second call with same texts: 0 additional calls to inner
        let res2 = embedder.embed(&texts).unwrap();
        assert_eq!(res2.len(), 2);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Third call with one new text: 1 additional call to inner (for the new text)
        let texts2 = vec!["query1".to_string(), "query3".to_string()];
        let res3 = embedder.embed(&texts2).unwrap();
        assert_eq!(res3.len(), 2);
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }
}
