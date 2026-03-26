use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

pub use crate::internal::config::Ignore;
use anyhow::{Result, anyhow};
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
    pub fn is_hybrid(&self) -> bool {
        matches!(self, Engine::Hybrid)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchPlan {
    pub name: String,
    pub query_expansion: QueryExpansionPolicy,
    pub retrievers: Vec<RetrieverPolicy>,
    pub fusion: FusionPolicy,
    pub reranking: RerankingPolicy,
}

impl SearchPlan {
    pub fn default_lexical() -> Self {
        Self {
            name: "lexical".to_string(),
            query_expansion: QueryExpansionPolicy::None,
            retrievers: vec![RetrieverPolicy::Bm25],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::None,
        }
    }

    pub fn categorize_score(&self, score: f64) -> ScoreConfidence {
        if score > 0.8 {
            ScoreConfidence::High
        } else if score > 0.4 {
            ScoreConfidence::Medium
        } else {
            ScoreConfidence::Low
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum QueryExpansionPolicy {
    None,
    Synonym,
    Hyde,
    Splade,
    Classified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum RetrieverPolicy {
    Bm25,
    Phrase,
    Vector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum FusionPolicy {
    Rrf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum RerankingPolicy {
    None,
    PositionAware,
    Llm,
    Jina,
    Gemma,
}

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub intent: Option<String>,
    pub path: PathBuf,
    pub strategy: String,
    pub limit: usize,
    pub shortlist: usize,
    pub verbose: u8,
    pub retrievers: Option<Vec<RetrieverPolicy>>,
    pub fusion: Option<FusionPolicy>,
    pub reranking: Option<RerankingPolicy>,
    pub dense_model: DenseModelSpec,
    pub rerank_model: Option<crate::search::adapters::qwen::QwenModelSpec>,
    pub gemma_model: Option<crate::search::adapters::gemma::GemmaModelSpec>,
    pub query_cache: Option<QueryEmbeddingCache>,
    pub cache_dir: Option<PathBuf>,
    pub telemetry: std::sync::Arc<crate::system::Telemetry>,
    pub prompts: Option<crate::config::PromptsConfig>,
}

impl SearchRequest {
    pub fn new(strategy: &str, query: impl Into<String>, path: PathBuf) -> Self {
        Self {
            query: query.into(),
            intent: None,
            path,
            strategy: strategy.to_string(),
            limit: 10,
            shortlist: 50,
            verbose: 0,
            retrievers: None,
            fusion: None,
            reranking: None,
            dense_model: DenseModelSpec::default(),
            rerank_model: None,
            gemma_model: None,
            query_cache: None,
            cache_dir: None,
            telemetry: std::sync::Arc::new(crate::system::Telemetry::new()),
            prompts: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub confidence: ScoreConfidence,
    pub location: Option<String>,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Clone)]
pub struct Bm25Index {
    pub doc_freq: HashMap<String, usize>,
    pub term_freqs: HashMap<String, HashMap<String, usize>>,
    pub doc_lengths: HashMap<String, usize>,
    pub avg_doc_len: f64,
    pub num_docs: usize,
}

impl Bm25Index {
    pub fn build(documents: &[Document]) -> Self {
        let mut doc_freq = HashMap::new();
        let mut term_freqs = HashMap::new();
        let mut doc_lengths = HashMap::new();
        let mut total_len = 0;

        for doc in documents {
            let terms: HashSet<_> = doc.terms.keys().collect();
            term_freqs.insert(doc.id.clone(), doc.terms.clone());
            doc_lengths.insert(doc.id.clone(), doc.length);
            total_len += doc.length;

            for term in terms {
                *doc_freq.entry(term.clone()).or_insert(0) += 1;
            }
        }

        let avg_doc_len = if documents.is_empty() {
            0.0
        } else {
            total_len as f64 / documents.len() as f64
        };

        Self {
            doc_freq,
            term_freqs,
            doc_lengths,
            avg_doc_len,
            num_docs: documents.len(),
        }
    }

    pub fn score(&self, query: &[String]) -> Vec<(String, f64)> {
        let mut scores = HashMap::new();
        let k1 = 1.2;
        let b = 0.75;

        for term in query {
            if let Some(&df) = self.doc_freq.get(term) {
                let idf = ((self.num_docs as f64 - df as f64 + 0.5) / (df as f64 + 0.5) + 1.0).ln();
                for (doc_id, terms) in &self.term_freqs {
                    let Some(&tf) = terms.get(term) else {
                        continue;
                    };
                    let doc_len = *self.doc_lengths.get(doc_id).unwrap_or(&0) as f64;
                    let tf = tf as f64;
                    let score = idf * (tf * (k1 + 1.0))
                        / (tf + k1 * (1.0 - b + b * doc_len / self.avg_doc_len));
                    *scores.entry(doc_id.clone()).or_insert(0.0) += score;
                }
            }
        }
        let mut results: Vec<_> = scores.into_iter().collect();
        results.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        results
    }
}

pub struct StrategyPreset {
    pub name: String,
    pub plan: SearchPlan,
}

pub struct StrategyPresetRegistry {
    presets: HashMap<String, SearchPlan>,
}

impl Default for StrategyPresetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyPresetRegistry {
    pub fn new() -> Self {
        Self {
            presets: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, plan: SearchPlan) {
        self.presets.insert(name.to_string(), plan);
    }

    pub fn resolve(&self, name: &str) -> Result<SearchPlan> {
        self.presets
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow!("strategy not found: {}", name))
    }

    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.presets.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn default_registry() -> Self {
        let mut registry = Self::new();

        let lexical_plan = SearchPlan {
            name: "lexical".to_string(),
            query_expansion: QueryExpansionPolicy::None,
            retrievers: vec![RetrieverPolicy::Bm25],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::None,
        };
        registry.register("lexical", lexical_plan.clone());
        registry.register(
            "bm25",
            SearchPlan {
                name: "bm25".to_string(),
                ..lexical_plan.clone()
            },
        );

        registry.register(
            "vector",
            SearchPlan {
                name: "vector".to_string(),
                query_expansion: QueryExpansionPolicy::None,
                retrievers: vec![RetrieverPolicy::Vector],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::None,
            },
        );

        let hybrid_plan = SearchPlan {
            name: "hybrid".to_string(),
            query_expansion: QueryExpansionPolicy::None,
            retrievers: vec![RetrieverPolicy::Bm25, RetrieverPolicy::Vector],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::None,
        };
        registry.register("hybrid", hybrid_plan);

        let page_index_hybrid_plan = SearchPlan {
            name: "page-index-hybrid".to_string(),
            query_expansion: QueryExpansionPolicy::Splade,
            retrievers: vec![
                RetrieverPolicy::Bm25,
                RetrieverPolicy::Phrase,
                RetrieverPolicy::Vector,
            ],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::PositionAware,
        };
        registry.register("page-index-hybrid", page_index_hybrid_plan.clone());
        registry.register(
            "legacy-hybrid",
            SearchPlan {
                name: "legacy-hybrid".to_string(),
                query_expansion: QueryExpansionPolicy::None,
                retrievers: page_index_hybrid_plan.retrievers.clone(),
                fusion: page_index_hybrid_plan.fusion,
                reranking: page_index_hybrid_plan.reranking,
            },
        );

        let page_index_llm_plan = SearchPlan {
            name: "page-index-llm".to_string(),
            query_expansion: QueryExpansionPolicy::Hyde,
            retrievers: vec![
                RetrieverPolicy::Bm25,
                RetrieverPolicy::Phrase,
                RetrieverPolicy::Vector,
            ],
            fusion: FusionPolicy::Rrf,
            reranking: RerankingPolicy::Llm,
        };
        registry.register("page-index-llm", page_index_llm_plan);

        // page-index-qwen (alias for page-index-llm for explicit Qwen testing)
        registry.register(
            "page-index-qwen",
            SearchPlan {
                name: "page-index-qwen".to_string(),
                query_expansion: QueryExpansionPolicy::None,
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                    RetrieverPolicy::Vector,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::Llm,
            },
        );

        // page-index-splade (generative expansion focus)
        registry.register(
            "page-index-splade",
            SearchPlan {
                name: "page-index-splade".to_string(),
                query_expansion: QueryExpansionPolicy::Splade,
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                    RetrieverPolicy::Vector,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::PositionAware,
            },
        );

        // page-index-classified (classification-driven expansion)
        registry.register(
            "page-index-classified",
            SearchPlan {
                name: "page-index-classified".to_string(),
                query_expansion: QueryExpansionPolicy::Classified,
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                    RetrieverPolicy::Vector,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::PositionAware,
            },
        );

        // page-index-jina (Jina Reranker v3)
        registry.register(
            "page-index-jina",
            SearchPlan {
                name: "page-index-jina".to_string(),
                query_expansion: QueryExpansionPolicy::Splade,
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                    RetrieverPolicy::Vector,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::Jina,
            },
        );

        // page-index-gemma (Gemma 3)
        registry.register(
            "page-index-gemma",
            SearchPlan {
                name: "page-index-gemma".to_string(),
                query_expansion: QueryExpansionPolicy::Splade,
                retrievers: vec![
                    RetrieverPolicy::Bm25,
                    RetrieverPolicy::Phrase,
                    RetrieverPolicy::Vector,
                ],
                fusion: FusionPolicy::Rrf,
                reranking: RerankingPolicy::Gemma,
            },
        );

        registry
    }
}

pub trait Retriever: Send + Sync {
    fn retrieve(
        &self,
        query: &[QueryVariant],
        corpus: &PreparedCorpus,
        limit: usize,
        verbose: u8,
    ) -> Result<CandidateList>;
    fn policy(&self) -> RetrieverPolicy;
}

pub trait Fuser: Send + Sync {
    fn fuse(&self, lists: &[CandidateList], limit: usize, verbose: u8) -> Result<CandidateList>;
}

pub trait Expander: Send + Sync {
    fn expand(&self, query: &str) -> Vec<QueryVariant>;
}

pub trait Reranker: Send + Sync {
    fn rerank(&self, query: &str, candidates: CandidateList, limit: usize)
    -> Result<CandidateList>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_generative(&self) -> Option<&dyn GenerativeModel> {
        None
    }
}

pub trait Conversation: Send + Sync {
    fn send(&mut self, message: &str, max_tokens: usize) -> Result<String>;
    fn history(&self) -> &[String];
}

pub trait GenerativeModel: Send + Sync {
    fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String>;
    fn start_conversation(&self) -> Result<Box<dyn Conversation>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateList {
    pub results: Vec<Candidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub path: std::path::PathBuf,
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

pub trait Embedder: Send + Sync {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
}

#[derive(Clone)]
pub struct CachedEmbedder {
    pub inner: Arc<dyn Embedder>,
    pub cache: QueryEmbeddingCache,
}

impl Embedder for CachedEmbedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut cached = Vec::with_capacity(texts.len());
        let mut missing_indices = Vec::new();
        let mut missing_texts = Vec::new();

        {
            let cache = self
                .cache
                .read()
                .map_err(|_| anyhow!("query embedding cache read lock poisoned"))?;

            for (index, text) in texts.iter().enumerate() {
                if let Some(embedding) = cache.get(text) {
                    cached.push(Some(embedding.clone()));
                } else {
                    cached.push(None);
                    missing_indices.push(index);
                    missing_texts.push(text.clone());
                }
            }
        }

        if !missing_texts.is_empty() {
            let computed = self.inner.embed(&missing_texts)?;
            if computed.len() != missing_texts.len() {
                return Err(anyhow!(
                    "embedder returned {} embeddings for {} inputs",
                    computed.len(),
                    missing_texts.len()
                ));
            }

            let mut cache = self
                .cache
                .write()
                .map_err(|_| anyhow!("query embedding cache write lock poisoned"))?;

            for (result_offset, original_index) in missing_indices.into_iter().enumerate() {
                let embedding = computed[result_offset].clone();
                let text = texts[original_index].clone();
                cache.insert(text, embedding.clone());
                cached[original_index] = Some(embedding);
            }
        }

        cached
            .into_iter()
            .map(|embedding| {
                embedding.ok_or_else(|| anyhow!("missing embedding after cache resolution"))
            })
            .collect()
    }

    fn dimension(&self) -> usize {
        self.inner.dimension()
    }
}

pub type QueryEmbeddingCache = Arc<std::sync::RwLock<HashMap<String, Vec<f32>>>>;

pub trait CorpusRepository: Send + Sync {
    fn load(
        &self,
        path: &std::path::Path,
        ignore: Option<&Ignore>,
        verbose: u8,
        embedder: Option<&dyn Embedder>,
        telemetry: &crate::system::Telemetry,
        cache_dir: Option<&std::path::Path>,
    ) -> Result<LoadedCorpus>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchTelemetry {
    pub heuristic_hits: usize,
    pub blob_hits: usize,
    pub embedding_hits: usize,
    pub total_files: usize,
    pub total_segments: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryVariant {
    pub text: String,
    pub weight: f64,
}

pub struct PreparedCorpus<'a> {
    pub documents: &'a [Document],
    pub bm25_index: Option<&'a Bm25Index>,
}

pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    Text,
}

#[cfg(test)]
mod tests {
    use super::{
        QueryExpansionPolicy, RerankingPolicy, RetrieverPolicy, StrategyPresetRegistry,
    };

    #[test]
    fn default_registry_includes_vector_strategy() {
        let plan = StrategyPresetRegistry::default_registry()
            .resolve("vector")
            .expect("vector preset should be registered");

        assert_eq!(plan.name, "vector");
        assert_eq!(plan.query_expansion, QueryExpansionPolicy::None);
        assert_eq!(plan.retrievers, vec![RetrieverPolicy::Vector]);
        assert_eq!(plan.reranking, RerankingPolicy::None);
    }
}
