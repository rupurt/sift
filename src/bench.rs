use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::system::{HardwareSummary, current_git_sha, detect_hardware_summary};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    Bm25,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BenchmarkMetadata {
    pub engine: Engine,
    pub command: String,
    pub git_sha: Option<String>,
    pub corpus_documents: usize,
    pub corpus_bytes: u64,
    pub hardware: HardwareSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityMetrics {
    pub ndcg_at_10: f64,
    pub mrr_at_10: f64,
    pub recall_at_10: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityBenchmarkReport {
    pub metadata: BenchmarkMetadata,
    pub metrics: QualityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LatencyMetrics {
    pub prepare_ms: f64,
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub max_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LatencyBenchmarkReport {
    pub metadata: BenchmarkMetadata,
    pub latency_ms: LatencyMetrics,
}

#[derive(Debug, Clone)]
pub struct QualityBenchmarkRequest {
    pub engine: Engine,
    pub command: String,
    pub corpus_dir: PathBuf,
    pub qrels_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LatencyBenchmarkRequest {
    pub engine: Engine,
    pub command: String,
    pub corpus_dir: PathBuf,
    pub queries_path: PathBuf,
}

pub fn run_quality_benchmark(request: &QualityBenchmarkRequest) -> Result<QualityBenchmarkReport> {
    ensure_bm25_only(request.engine)?;

    let prepare_started = Instant::now();
    let corpus = load_corpus(&request.corpus_dir)?;
    let index = Bm25Index::build(&corpus.documents);
    let queries_path = request.corpus_dir.join("test-queries.tsv");
    let queries = load_queries(&queries_path)?;
    let qrels = load_qrels(&request.qrels_path)?;
    let _prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

    let mut ndcg_total = 0.0;
    let mut mrr_total = 0.0;
    let mut recall_total = 0.0;
    let mut counted_queries = 0_usize;

    for (query_id, relevances) in &qrels {
        let query_text = queries
            .get(query_id)
            .with_context(|| format!("missing query text for qrels query-id '{query_id}'"))?;
        let ranked = index.score(query_text);
        let top_ten = &ranked[..ranked.len().min(10)];

        ndcg_total += ndcg_at_10(top_ten, relevances);
        mrr_total += mrr_at_10(top_ten, relevances);
        recall_total += recall_at_10(top_ten, relevances);
        counted_queries += 1;
    }

    if counted_queries == 0 {
        bail!("quality benchmark requires at least one qrels row");
    }

    Ok(QualityBenchmarkReport {
        metadata: BenchmarkMetadata {
            engine: request.engine,
            command: request.command.clone(),
            git_sha: current_git_sha(),
            corpus_documents: corpus.documents.len(),
            corpus_bytes: corpus.total_bytes,
            hardware: detect_hardware_summary(),
        },
        metrics: QualityMetrics {
            ndcg_at_10: ndcg_total / counted_queries as f64,
            mrr_at_10: mrr_total / counted_queries as f64,
            recall_at_10: recall_total / counted_queries as f64,
        },
    })
}

pub fn run_latency_benchmark(request: &LatencyBenchmarkRequest) -> Result<LatencyBenchmarkReport> {
    ensure_bm25_only(request.engine)?;

    let prepare_started = Instant::now();
    let corpus = load_corpus(&request.corpus_dir)?;
    let index = Bm25Index::build(&corpus.documents);
    let queries = load_queries(&request.queries_path)?;
    let prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

    if queries.is_empty() {
        bail!("latency benchmark requires at least one query");
    }

    let mut timings = Vec::with_capacity(queries.len());
    for query_text in queries.values() {
        let started = Instant::now();
        let _ = index.score(query_text);
        timings.push(started.elapsed().as_secs_f64() * 1000.0);
    }

    timings.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

    Ok(LatencyBenchmarkReport {
        metadata: BenchmarkMetadata {
            engine: request.engine,
            command: request.command.clone(),
            git_sha: current_git_sha(),
            corpus_documents: corpus.documents.len(),
            corpus_bytes: corpus.total_bytes,
            hardware: detect_hardware_summary(),
        },
        latency_ms: LatencyMetrics {
            prepare_ms,
            p50_ms: percentile(&timings, 0.50),
            p90_ms: percentile(&timings, 0.90),
            max_ms: timings.last().copied().unwrap_or(0.0),
        },
    })
}

fn ensure_bm25_only(engine: Engine) -> Result<()> {
    if engine == Engine::Hybrid {
        bail!("hybrid benchmarks are not available until story 1vzJfv000");
    }

    Ok(())
}

struct LoadedCorpus {
    documents: Vec<Document>,
    total_bytes: u64,
}

#[derive(Debug, Clone)]
struct Document {
    id: String,
    length: usize,
    terms: HashMap<String, usize>,
}

struct Bm25Index {
    documents: Vec<Document>,
    doc_freq: HashMap<String, usize>,
    avg_doc_len: f64,
}

impl Bm25Index {
    fn build(documents: &[Document]) -> Self {
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

    fn score(&self, query: &str) -> Vec<(String, f64)> {
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

                let doc_freq = self.doc_freq.get(term).copied().unwrap_or(0) as f64;
                let idf = ((total_docs - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln();
                let length_ratio = if self.avg_doc_len > 0.0 {
                    document.length as f64 / self.avg_doc_len
                } else {
                    1.0
                };
                let denominator = tf + 1.5 * (1.0 - 0.75 + 0.75 * length_ratio);
                score += idf * (tf * (1.5 + 1.0) / denominator);
            }

            ranked.push((document.id.clone(), score));
        }

        ranked.sort_by(|left, right| {
            right
                .1
                .partial_cmp(&left.1)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.0.cmp(&right.0))
        });
        ranked
    }
}

fn load_corpus(corpus_dir: &Path) -> Result<LoadedCorpus> {
    let mut documents = Vec::new();
    let mut total_bytes = 0_u64;

    for entry in WalkDir::new(corpus_dir).max_depth(1) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("txt") {
            continue;
        }

        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read corpus document {}", path.display()))?;
        total_bytes += contents.len() as u64;
        let terms = tokenize(&contents)
            .into_iter()
            .fold(HashMap::new(), |mut acc, term| {
                *acc.entry(term).or_insert(0) += 1;
                acc
            });
        let length = terms.values().sum();
        let id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .with_context(|| format!("invalid document filename {}", path.display()))?
            .to_string();

        documents.push(Document { id, length, terms });
    }

    documents.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(LoadedCorpus {
        documents,
        total_bytes,
    })
}

fn load_queries(path: &Path) -> Result<HashMap<String, String>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read queries file {}", path.display()))?;
    let mut queries = HashMap::new();

    for (index, line) in contents.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if index == 0 && line.starts_with("query-id\t") {
            continue;
        }

        let mut columns = line.splitn(2, '\t');
        let query_id = columns
            .next()
            .filter(|value| !value.is_empty())
            .with_context(|| format!("missing query id in {}", path.display()))?;
        let query_text = columns
            .next()
            .filter(|value| !value.is_empty())
            .with_context(|| format!("missing query text in {}", path.display()))?;

        queries.insert(query_id.to_string(), query_text.to_string());
    }

    Ok(queries)
}

fn load_qrels(path: &Path) -> Result<HashMap<String, HashMap<String, u32>>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read qrels file {}", path.display()))?;
    let mut qrels = HashMap::new();

    for (index, line) in contents.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if index == 0 && line.starts_with("query-id\t") {
            continue;
        }

        let mut columns = line.split('\t');
        let query_id = columns
            .next()
            .filter(|value| !value.is_empty())
            .with_context(|| format!("missing qrels query id in {}", path.display()))?;
        let corpus_id = columns
            .next()
            .filter(|value| !value.is_empty())
            .with_context(|| format!("missing qrels corpus id in {}", path.display()))?;
        let score = columns
            .next()
            .with_context(|| format!("missing qrels score in {}", path.display()))?
            .parse::<u32>()
            .with_context(|| format!("invalid qrels score in {}", path.display()))?;

        qrels
            .entry(query_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(corpus_id.to_string(), score);
    }

    Ok(qrels)
}

fn tokenize(text: &str) -> Vec<String> {
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

fn mrr_at_10(results: &[(String, f64)], relevances: &HashMap<String, u32>) -> f64 {
    results
        .iter()
        .enumerate()
        .find(|(_, (doc_id, _))| relevances.get(doc_id).copied().unwrap_or(0) > 0)
        .map(|(index, _)| 1.0 / (index as f64 + 1.0))
        .unwrap_or(0.0)
}

fn recall_at_10(results: &[(String, f64)], relevances: &HashMap<String, u32>) -> f64 {
    let relevant_total = relevances.values().filter(|score| **score > 0).count();
    if relevant_total == 0 {
        return 0.0;
    }

    let hits = results
        .iter()
        .filter(|(doc_id, _)| relevances.get(doc_id).copied().unwrap_or(0) > 0)
        .count();

    hits as f64 / relevant_total as f64
}

fn ndcg_at_10(results: &[(String, f64)], relevances: &HashMap<String, u32>) -> f64 {
    let dcg = results
        .iter()
        .enumerate()
        .map(|(index, (doc_id, _))| {
            discounted_gain(index, relevances.get(doc_id).copied().unwrap_or(0))
        })
        .sum::<f64>();

    let mut ideal = relevances.values().copied().collect::<Vec<_>>();
    ideal.sort_by(|left, right| right.cmp(left));
    let idcg = ideal
        .iter()
        .take(10)
        .enumerate()
        .map(|(index, score)| discounted_gain(index, *score))
        .sum::<f64>();

    if idcg == 0.0 { 0.0 } else { dcg / idcg }
}

fn discounted_gain(index: usize, relevance: u32) -> f64 {
    if relevance == 0 {
        return 0.0;
    }

    let gain = 2_f64.powi(relevance as i32) - 1.0;
    let rank = index as f64 + 2.0;
    gain / rank.log2()
}

fn percentile(values: &[f64], quantile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let raw_index = (values.len() as f64 * quantile).ceil() as usize;
    let index = raw_index.saturating_sub(1).min(values.len() - 1);
    values[index]
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{
        Engine, LatencyBenchmarkRequest, QualityBenchmarkRequest, run_latency_benchmark,
        run_quality_benchmark,
    };

    #[test]
    fn quality_benchmark_report_contains_metadata_and_metrics() {
        let corpus_dir = sample_corpus_dir();
        let request = QualityBenchmarkRequest {
            engine: Engine::Bm25,
            command: "sift bench quality --engine bm25".to_string(),
            corpus_dir: corpus_dir.path().to_path_buf(),
            qrels_path: corpus_dir.path().join("qrels/test.tsv"),
        };

        let report = run_quality_benchmark(&request).expect("quality report");
        assert_eq!(report.metadata.engine, Engine::Bm25);
        assert_eq!(report.metadata.corpus_documents, 2);
        assert!(!report.metadata.command.is_empty());
        assert!(report.metrics.ndcg_at_10 >= 0.0);
        assert!(report.metrics.mrr_at_10 >= 0.0);
        assert!(report.metrics.recall_at_10 >= 0.0);
    }

    #[test]
    fn latency_benchmark_report_contains_reproducible_fields() {
        let corpus_dir = sample_corpus_dir();
        let request = LatencyBenchmarkRequest {
            engine: Engine::Bm25,
            command: "sift bench latency --engine bm25".to_string(),
            corpus_dir: corpus_dir.path().to_path_buf(),
            queries_path: corpus_dir.path().join("test-queries.tsv"),
        };

        let report = run_latency_benchmark(&request).expect("latency report");
        assert_eq!(report.metadata.engine, Engine::Bm25);
        assert_eq!(report.metadata.corpus_documents, 2);
        assert!(!report.metadata.command.is_empty());
        assert!(report.latency_ms.prepare_ms >= 0.0);
        assert!(report.latency_ms.p50_ms >= 0.0);
        assert!(report.latency_ms.p90_ms >= report.latency_ms.p50_ms);
        assert!(report.latency_ms.max_ms >= report.latency_ms.p90_ms);
    }

    fn sample_corpus_dir() -> tempfile::TempDir {
        let dir = tempdir().expect("corpus dir");
        fs::write(
            dir.path().join("doc-a.txt"),
            "Alpha\n\nrust search benchmark corpus",
        )
        .expect("write doc a");
        fs::write(
            dir.path().join("doc-b.txt"),
            "Beta\n\nsemantic rerank later story",
        )
        .expect("write doc b");
        fs::write(
            dir.path().join("test-queries.tsv"),
            "query-id\ttext\nq-1\trust benchmark\nq-2\trerank story\n",
        )
        .expect("write queries");
        fs::create_dir_all(dir.path().join("qrels")).expect("qrels dir");
        fs::write(
            dir.path().join("qrels/test.tsv"),
            "query-id\tcorpus-id\tscore\nq-1\tdoc-a\t1\nq-2\tdoc-b\t1\n",
        )
        .expect("write qrels");
        dir
    }
}
