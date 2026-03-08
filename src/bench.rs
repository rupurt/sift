use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::dense::{DenseModelSpec, DenseReranker};
use crate::search::{Bm25Index, Engine, RankedDocument, load_materialized_corpus, rank_corpus};
use crate::system::{HardwareSummary, current_git_sha, detect_hardware_summary};

const LATENCY_TARGET_MS: f64 = 200.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BenchmarkMetadata {
    pub engine: Engine,
    pub baseline_engine: Option<Engine>,
    pub command: String,
    pub git_sha: Option<String>,
    pub corpus_documents: usize,
    pub corpus_bytes: u64,
    pub hardware: HardwareSummary,
    pub dense_model: Option<String>,
    pub dense_revision: Option<String>,
    pub dense_max_length: Option<usize>,
    pub shortlist: Option<usize>,
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
    pub baseline_metrics: Option<QualityMetrics>,
    pub delta: Option<QualityMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LatencyMetrics {
    pub prepare_ms: f64,
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub max_ms: f64,
    pub target_ms: f64,
    pub p50_over_target_ms: f64,
    pub p90_over_target_ms: f64,
    pub max_over_target_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LatencyBenchmarkReport {
    pub metadata: BenchmarkMetadata,
    pub latency_ms: LatencyMetrics,
}

#[derive(Debug, Clone)]
pub struct QualityBenchmarkRequest {
    pub engine: Engine,
    pub baseline: Option<Engine>,
    pub command: String,
    pub corpus_dir: PathBuf,
    pub qrels_path: PathBuf,
    pub shortlist: usize,
    pub dense_model: DenseModelSpec,
}

#[derive(Debug, Clone)]
pub struct LatencyBenchmarkRequest {
    pub engine: Engine,
    pub command: String,
    pub corpus_dir: PathBuf,
    pub queries_path: PathBuf,
    pub shortlist: usize,
    pub dense_model: DenseModelSpec,
}

pub fn run_quality_benchmark(request: &QualityBenchmarkRequest) -> Result<QualityBenchmarkReport> {
    let prepare_started = Instant::now();
    let corpus = load_materialized_corpus(&request.corpus_dir)?;
    let index = Bm25Index::build(&corpus.documents);
    let dense = match request.engine {
        Engine::Bm25 => None,
        Engine::Hybrid => Some(DenseReranker::load(request.dense_model.clone())?),
    };
    let queries_path = request.corpus_dir.join("test-queries.tsv");
    let queries = load_queries(&queries_path)?;
    let qrels = load_qrels(&request.qrels_path)?;
    let _prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

    let metrics = evaluate_quality(
        &corpus,
        &index,
        dense.as_ref(),
        &queries,
        &qrels,
        request.engine,
        request.shortlist,
    )?;

    let baseline_engine = match request.engine {
        Engine::Bm25 => None,
        Engine::Hybrid => Some(request.baseline.unwrap_or(Engine::Bm25)),
    };
    let baseline_metrics = match baseline_engine {
        Some(Engine::Bm25) => Some(evaluate_quality(
            &corpus,
            &index,
            None,
            &queries,
            &qrels,
            Engine::Bm25,
            request.shortlist,
        )?),
        Some(engine) => bail!(
            "unsupported quality benchmark baseline engine '{:?}'",
            engine
        ),
        None => None,
    };
    let delta = baseline_metrics
        .as_ref()
        .map(|baseline| quality_delta(&metrics, baseline));

    Ok(QualityBenchmarkReport {
        metadata: BenchmarkMetadata {
            engine: request.engine,
            baseline_engine,
            command: request.command.clone(),
            git_sha: current_git_sha(),
            corpus_documents: corpus.documents.len(),
            corpus_bytes: corpus.total_bytes,
            hardware: detect_hardware_summary(),
            dense_model: dense
                .as_ref()
                .map(|reranker| reranker.model_id().to_string()),
            dense_revision: dense
                .as_ref()
                .map(|reranker| reranker.revision().to_string()),
            dense_max_length: dense.as_ref().map(DenseReranker::max_length),
            shortlist: matches!(request.engine, Engine::Hybrid).then_some(request.shortlist),
        },
        metrics,
        baseline_metrics,
        delta,
    })
}

pub fn run_latency_benchmark(request: &LatencyBenchmarkRequest) -> Result<LatencyBenchmarkReport> {
    let prepare_started = Instant::now();
    let corpus = load_materialized_corpus(&request.corpus_dir)?;
    let index = Bm25Index::build(&corpus.documents);
    let dense = match request.engine {
        Engine::Bm25 => None,
        Engine::Hybrid => Some(DenseReranker::load(request.dense_model.clone())?),
    };
    let queries = load_queries(&request.queries_path)?;
    let prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

    if queries.is_empty() {
        bail!("latency benchmark requires at least one query");
    }

    let mut timings = Vec::with_capacity(queries.len());
    for query_text in queries.values() {
        let started = Instant::now();
        let _ = rank_corpus(
            &corpus,
            &index,
            dense.as_ref(),
            query_text,
            request.engine,
            10,
            request.shortlist,
        )?;
        timings.push(started.elapsed().as_secs_f64() * 1000.0);
    }

    timings.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    let p50_ms = percentile(&timings, 0.50);
    let p90_ms = percentile(&timings, 0.90);
    let max_ms = timings.last().copied().unwrap_or(0.0);

    Ok(LatencyBenchmarkReport {
        metadata: BenchmarkMetadata {
            engine: request.engine,
            baseline_engine: None,
            command: request.command.clone(),
            git_sha: current_git_sha(),
            corpus_documents: corpus.documents.len(),
            corpus_bytes: corpus.total_bytes,
            hardware: detect_hardware_summary(),
            dense_model: dense
                .as_ref()
                .map(|reranker| reranker.model_id().to_string()),
            dense_revision: dense
                .as_ref()
                .map(|reranker| reranker.revision().to_string()),
            dense_max_length: dense.as_ref().map(DenseReranker::max_length),
            shortlist: matches!(request.engine, Engine::Hybrid).then_some(request.shortlist),
        },
        latency_ms: LatencyMetrics {
            prepare_ms,
            p50_ms,
            p90_ms,
            max_ms,
            target_ms: LATENCY_TARGET_MS,
            p50_over_target_ms: over_target_ms(p50_ms),
            p90_over_target_ms: over_target_ms(p90_ms),
            max_over_target_ms: over_target_ms(max_ms),
        },
    })
}

fn evaluate_quality(
    corpus: &crate::search::LoadedCorpus,
    index: &Bm25Index,
    dense: Option<&DenseReranker>,
    queries: &HashMap<String, String>,
    qrels: &HashMap<String, HashMap<String, u32>>,
    engine: Engine,
    shortlist: usize,
) -> Result<QualityMetrics> {
    let mut ndcg_total = 0.0;
    let mut mrr_total = 0.0;
    let mut recall_total = 0.0;
    let mut counted_queries = 0_usize;

    for (query_id, relevances) in qrels {
        let query_text = queries
            .get(query_id)
            .with_context(|| format!("missing query text for qrels query-id '{query_id}'"))?;
        let ranked = rank_corpus(corpus, index, dense, query_text, engine, 10, shortlist)?;

        ndcg_total += ndcg_at_10(&ranked, relevances);
        mrr_total += mrr_at_10(&ranked, relevances);
        recall_total += recall_at_10(&ranked, relevances);
        counted_queries += 1;
    }

    if counted_queries == 0 {
        bail!("quality benchmark requires at least one qrels row");
    }

    Ok(QualityMetrics {
        ndcg_at_10: ndcg_total / counted_queries as f64,
        mrr_at_10: mrr_total / counted_queries as f64,
        recall_at_10: recall_total / counted_queries as f64,
    })
}

fn quality_delta(metrics: &QualityMetrics, baseline: &QualityMetrics) -> QualityMetrics {
    QualityMetrics {
        ndcg_at_10: metrics.ndcg_at_10 - baseline.ndcg_at_10,
        mrr_at_10: metrics.mrr_at_10 - baseline.mrr_at_10,
        recall_at_10: metrics.recall_at_10 - baseline.recall_at_10,
    }
}

fn over_target_ms(value: f64) -> f64 {
    (value - LATENCY_TARGET_MS).max(0.0)
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

fn mrr_at_10(results: &[RankedDocument], relevances: &HashMap<String, u32>) -> f64 {
    results
        .iter()
        .enumerate()
        .find(|(_, result)| relevances.get(&result.id).copied().unwrap_or(0) > 0)
        .map(|(index, _)| 1.0 / (index as f64 + 1.0))
        .unwrap_or(0.0)
}

fn recall_at_10(results: &[RankedDocument], relevances: &HashMap<String, u32>) -> f64 {
    let relevant_total = relevances.values().filter(|score| **score > 0).count();
    if relevant_total == 0 {
        return 0.0;
    }

    let hits = results
        .iter()
        .filter(|result| relevances.get(&result.id).copied().unwrap_or(0) > 0)
        .count();

    hits as f64 / relevant_total as f64
}

fn ndcg_at_10(results: &[RankedDocument], relevances: &HashMap<String, u32>) -> f64 {
    let dcg = results
        .iter()
        .enumerate()
        .map(|(index, result)| {
            discounted_gain(index, relevances.get(&result.id).copied().unwrap_or(0))
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
mod quality {
    use std::fs;

    use tempfile::tempdir;

    use crate::dense::DenseModelSpec;
    use crate::search::DEFAULT_HYBRID_SHORTLIST;

    use super::{Engine, QualityBenchmarkRequest, quality_delta, run_quality_benchmark};

    #[test]
    fn quality_benchmark_report_contains_metadata_and_metrics() {
        let corpus_dir = sample_corpus_dir();
        let request = QualityBenchmarkRequest {
            engine: Engine::Bm25,
            baseline: None,
            command: "sift bench quality --engine bm25".to_string(),
            corpus_dir: corpus_dir.path().to_path_buf(),
            qrels_path: corpus_dir.path().join("qrels/test.tsv"),
            shortlist: DEFAULT_HYBRID_SHORTLIST,
            dense_model: DenseModelSpec::default(),
        };

        let report = run_quality_benchmark(&request).expect("quality report");
        assert_eq!(report.metadata.engine, Engine::Bm25);
        assert_eq!(report.metadata.corpus_documents, 2);
        assert!(!report.metadata.command.is_empty());
        assert!(report.metrics.ndcg_at_10 >= 0.0);
        assert!(report.metrics.mrr_at_10 >= 0.0);
        assert!(report.metrics.recall_at_10 >= 0.0);
        assert!(report.delta.is_none());
    }

    #[test]
    fn quality_delta_is_reported_as_exact_difference() {
        let delta = quality_delta(
            &super::QualityMetrics {
                ndcg_at_10: 0.72,
                mrr_at_10: 0.61,
                recall_at_10: 0.80,
            },
            &super::QualityMetrics {
                ndcg_at_10: 0.70,
                mrr_at_10: 0.60,
                recall_at_10: 0.78,
            },
        );

        assert!((delta.ndcg_at_10 - 0.02).abs() < 1e-9);
        assert!((delta.mrr_at_10 - 0.01).abs() < 1e-9);
        assert!((delta.recall_at_10 - 0.02).abs() < 1e-9);
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

#[cfg(test)]
mod latency {
    use std::fs;

    use tempfile::tempdir;

    use crate::dense::DenseModelSpec;
    use crate::search::DEFAULT_HYBRID_SHORTLIST;

    use super::{Engine, LatencyBenchmarkRequest, over_target_ms, run_latency_benchmark};

    #[test]
    fn latency_benchmark_report_contains_reproducible_fields() {
        let corpus_dir = sample_corpus_dir();
        let request = LatencyBenchmarkRequest {
            engine: Engine::Bm25,
            command: "sift bench latency --engine bm25".to_string(),
            corpus_dir: corpus_dir.path().to_path_buf(),
            queries_path: corpus_dir.path().join("test-queries.tsv"),
            shortlist: DEFAULT_HYBRID_SHORTLIST,
            dense_model: DenseModelSpec::default(),
        };

        let report = run_latency_benchmark(&request).expect("latency report");
        assert_eq!(report.metadata.engine, Engine::Bm25);
        assert_eq!(report.metadata.corpus_documents, 2);
        assert!(!report.metadata.command.is_empty());
        assert!(report.latency_ms.prepare_ms >= 0.0);
        assert!(report.latency_ms.p50_ms >= 0.0);
        assert!(report.latency_ms.p90_ms >= report.latency_ms.p50_ms);
        assert!(report.latency_ms.max_ms >= report.latency_ms.p90_ms);
        assert_eq!(report.latency_ms.target_ms, 200.0);
    }

    #[test]
    fn latency_shortfall_is_explicit_and_non_negative() {
        assert_eq!(over_target_ms(150.0), 0.0);
        assert_eq!(over_target_ms(200.0), 0.0);
        assert_eq!(over_target_ms(245.5), 45.5);
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
