use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, anyhow, bail};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};

use crate::config::Ignore;
use crate::dense::{DenseModelSpec, DenseReranker};
use crate::search::{LoadedCorpus, SearchRequest, load_search_corpus};
use crate::system::{HardwareSummary, current_git_sha, detect_hardware_summary};

const LATENCY_TARGET_MS: f64 = 200.0;

// --- DATASET MANAGEMENT ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadSummary {
    pub dataset: String,
    pub corpus_archive: String,
    pub queries_archive: String,
    pub qrels_test: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaterializationSummary {
    pub dataset: String,
    pub documents: usize,
    pub test_queries: usize,
    pub output_dir: String,
}

pub fn download_scifact_dataset(
    base_url: &str,
    qrels_base_url: &str,
    out_dir: &Path,
) -> Result<DownloadSummary> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("create download dir {}", out_dir.display()))?;
    fs::create_dir_all(out_dir.join("qrels"))
        .with_context(|| format!("create qrels dir {}", out_dir.join("qrels").display()))?;

    download_asset(
        &join_url(base_url, "corpus.jsonl.gz"),
        &out_dir.join("corpus.jsonl.gz"),
    )?;
    download_asset(
        &join_url(base_url, "queries.jsonl.gz"),
        &out_dir.join("queries.jsonl.gz"),
    )?;
    download_asset(
        &join_url(qrels_base_url, "test.tsv"),
        &out_dir.join("qrels").join("test.tsv"),
    )?;

    Ok(DownloadSummary {
        dataset: "scifact".to_string(),
        corpus_archive: "corpus.jsonl.gz".to_string(),
        queries_archive: "queries.jsonl.gz".to_string(),
        qrels_test: "qrels/test.tsv".to_string(),
    })
}

pub fn materialize_scifact_dir(
    source_dir: &Path,
    out_dir: &Path,
) -> Result<MaterializationSummary> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("create materialized dir {}", out_dir.display()))?;
    fs::create_dir_all(out_dir.join("qrels")).with_context(|| {
        format!(
            "create materialized qrels dir {}",
            out_dir.join("qrels").display()
        )
    })?;

    let qrels_path = source_dir.join("qrels").join("test.tsv");
    let qrel_rows = read_qrels(&qrels_path)?;
    let wanted_query_ids: HashSet<_> = qrel_rows.iter().map(|row| row.query_id.clone()).collect();

    let corpus_records = read_jsonl_gz::<CorpusRecord>(&source_dir.join("corpus.jsonl.gz"))?;
    let queries = read_jsonl_gz::<QueryRecord>(&source_dir.join("queries.jsonl.gz"))?;

    let mut documents = 0;
    for record in corpus_records {
        let filename = out_dir.join(format!("{}.txt", sanitize_doc_id(&record.id)));
        let mut body = String::new();
        if let Some(title) = record
            .title
            .as_deref()
            .filter(|title| !title.trim().is_empty())
        {
            body.push_str(title.trim());
            body.push_str("\n\n");
        }
        body.push_str(record.text.trim());

        fs::write(&filename, body)
            .with_context(|| format!("write materialized document {}", filename.display()))?;
        documents += 1;
    }

    let mut query_file = fs::File::create(out_dir.join("test-queries.tsv")).with_context(|| {
        format!(
            "create query file {}",
            out_dir.join("test-queries.tsv").display()
        )
    })?;
    writeln!(query_file, "query-id\ttext").context("write query header")?;

    let mut test_queries = 0;
    for query in queries {
        if wanted_query_ids.contains(&query.id) {
            writeln!(
                query_file,
                "{}\t{}",
                query.id,
                query.text.replace('\n', " ")
            )
            .context("write materialized query row")?;
            test_queries += 1;
        }
    }

    fs::copy(&qrels_path, out_dir.join("qrels").join("test.tsv")).with_context(|| {
        format!(
            "copy qrels {} -> {}",
            qrels_path.display(),
            out_dir.join("qrels").join("test.tsv").display()
        )
    })?;

    Ok(MaterializationSummary {
        dataset: "scifact".to_string(),
        documents,
        test_queries,
        output_dir: out_dir.display().to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct CorpusRecord {
    #[serde(rename = "_id")]
    id: String,
    #[serde(default)]
    title: Option<String>,
    text: String,
}

#[derive(Debug, Deserialize)]
struct QueryRecord {
    #[serde(rename = "_id")]
    id: String,
    text: String,
}

#[derive(Debug)]
struct QrelRow {
    query_id: String,
}

fn download_asset(url: &str, path: &Path) -> Result<()> {
    let mut response = ureq::get(url)
        .call()
        .with_context(|| format!("download {}", url))?;
    let bytes = response
        .body_mut()
        .read_to_vec()
        .with_context(|| format!("read response body from {}", url))?;

    fs::write(path, bytes).with_context(|| format!("write asset {}", path.display()))?;
    Ok(())
}

fn join_url(base: &str, file: &str) -> String {
    format!("{}/{}", base.trim_end_matches('/'), file)
}

fn read_jsonl_gz<T>(path: &Path) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de>,
{
    let file = fs::File::open(path).with_context(|| format!("open archive {}", path.display()))?;
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);
    let mut rows = Vec::new();

    for line in reader.lines() {
        let line = line.with_context(|| format!("read line from {}", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }
        let row = serde_json::from_str(&line)
            .with_context(|| format!("parse jsonl row from {}", path.display()))?;
        rows.push(row);
    }

    Ok(rows)
}

fn read_qrels(path: &Path) -> Result<Vec<QrelRow>> {
    let file = fs::File::open(path).with_context(|| format!("open qrels {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("read qrels line from {}", path.display()))?;
        if index == 0 || line.trim().is_empty() {
            continue;
        }

        let mut parts = line.split('\t');
        let query_id = parts
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("missing query-id in {}", path.display()))?;
        let _corpus_id = parts
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("missing corpus-id in {}", path.display()))?;
        let _score = parts
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("missing score in {}", path.display()))?;

        rows.push(QrelRow {
            query_id: query_id.to_string(),
        });
    }

    Ok(rows)
}

fn sanitize_doc_id(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '\0' => '_',
            other => other,
        })
        .collect()
}

// --- EVALUATION HARNESS ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationMetadata {
    pub strategy: String,
    pub plan: crate::search::SearchPlan,
    pub baseline_strategy: Option<String>,
    pub champion_strategy: Option<String>,
    pub command: String,
    pub git_sha: Option<String>,
    pub corpus_documents: usize,
    pub corpus_bytes: u64,
    pub segment_strategy: String,
    pub segment_count: usize,
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
pub struct QualityEvaluationReport {
    pub metadata: EvaluationMetadata,
    pub metrics: QualityMetrics,
    pub baseline_metrics: Option<QualityMetrics>,
    pub champion_metrics: Option<QualityMetrics>,
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
pub struct LatencyEvaluationReport {
    pub metadata: EvaluationMetadata,
    pub latency_ms: LatencyMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComparativeEvaluationReport {
    pub metadata: Vec<EvaluationMetadata>,
    pub results: Vec<StrategyComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyComparison {
    pub strategy: String,
    pub quality: QualityMetrics,
    pub latency: LatencyMetrics,
    pub telemetry: Option<crate::search::SearchTelemetry>,
}

#[derive(Debug, Clone)]
pub struct QualityEvaluationRequest {
    pub strategy: String,
    pub baseline: Option<String>,
    pub command: String,
    pub corpus_dir: PathBuf,
    pub queries_path: Option<PathBuf>,
    pub qrels_path: PathBuf,
    pub shortlist: usize,
    pub dense_model: DenseModelSpec,
    pub verbose: u8,
    pub query_limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct LatencyEvaluationRequest {
    pub strategy: String,
    pub command: String,
    pub corpus_dir: PathBuf,
    pub queries_path: PathBuf,
    pub shortlist: usize,
    pub dense_model: DenseModelSpec,
    pub verbose: u8,
    pub query_limit: Option<usize>,
}

pub fn run_quality_evaluation(
    request: &QualityEvaluationRequest,
    ignore: Option<&Ignore>,
) -> Result<QualityEvaluationReport> {
    let prepare_started = Instant::now();
    tracing::info!("→ loading dense model: {}", request.dense_model.model_id);
    let dense_for_load = std::sync::Arc::new(DenseReranker::load(request.dense_model.clone())?);
    let telemetry_for_load = std::sync::Arc::new(crate::system::Telemetry::new());
    let query_cache = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let corpus = load_search_corpus(&request.corpus_dir, ignore, request.verbose, Some(dense_for_load.as_ref()), &telemetry_for_load, None)?;
    let index = crate::search::Bm25Index::build(&corpus.documents);
    let queries_path = request
        .queries_path
        .clone()
        .unwrap_or_else(|| request.corpus_dir.join("test-queries.tsv"));
    let queries = load_queries(&queries_path)?;
    let qrels = load_qrels(&request.qrels_path)?;
    let _prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

    let registry = crate::search::StrategyPresetRegistry::default_registry();
    let champion_plan = registry.resolve("hybrid")?;
    let champion_strategy_name = champion_plan.name.clone();

    let env = crate::search::SearchEnvironment::new(
        &SearchRequest {
            strategy: request.strategy.clone(),
            query: String::new(),
            path: request.corpus_dir.clone(),
            limit: request.shortlist,
            shortlist: request.shortlist,
            dense_model: request.dense_model.clone(),
            verbose: request.verbose,
            retrievers: None,
            fusion: None,
            reranking: None,
            telemetry: telemetry_for_load.clone(),
            cache_dir: None,
            query_cache: Some(query_cache.clone()),
        },
        &corpus,
        &index,
        Some(dense_for_load.clone()),
    )?;

    let (metrics, _telemetry) = evaluate_quality(
        &queries,
        &qrels,
        &env,
        request.verbose,
        request.query_limit,
    )?;

    let baseline_strategy = request
        .baseline
        .clone()
        .or_else(|| Some("bm25".to_string()));
    let baseline_metrics = match &baseline_strategy {
        Some(strategy) => {
            let baseline_env = crate::search::SearchEnvironment::new(
                &SearchRequest {
                    strategy: strategy.clone(),
                    query: String::new(),
                    path: request.corpus_dir.clone(),
                    limit: request.shortlist,
                    shortlist: request.shortlist,
                    dense_model: request.dense_model.clone(),
                    verbose: request.verbose,
                    retrievers: None,
                    fusion: None,
                    reranking: None,
                    telemetry: telemetry_for_load.clone(),
                    cache_dir: None,
                    query_cache: Some(query_cache.clone()),
                },
                &corpus,
                &index,
                Some(dense_for_load.clone()),
            )?;
            let (m, _) = evaluate_quality(
                &queries,
                &qrels,
                &baseline_env,
                request.verbose,
                request.query_limit,
            )?;
            Some(m)
        },
        None => None,
    };

    let champion_metrics = if champion_strategy_name != request.strategy {
        let champion_env = crate::search::SearchEnvironment::new(
            &SearchRequest {
                strategy: champion_strategy_name.clone(),
                query: String::new(),
                path: request.corpus_dir.clone(),
                limit: request.shortlist,
                shortlist: request.shortlist,
                dense_model: request.dense_model.clone(),
                verbose: request.verbose,
                retrievers: None,
                fusion: None,
                reranking: None,
                telemetry: telemetry_for_load.clone(),
                cache_dir: None,
                query_cache: Some(query_cache.clone()),
            },
            &corpus,
            &index,
            Some(dense_for_load.clone()),
        )?;
        let (m, _) = evaluate_quality(
            &queries,
            &qrels,
            &champion_env,
            request.verbose,
            request.query_limit,
        )?;
        Some(m)
    } else {
        Some(metrics.clone())
    };

    let delta = baseline_metrics
        .as_ref()
        .map(|baseline| quality_delta(&metrics, baseline));

    Ok(QualityEvaluationReport {
        metadata: build_metadata(
            &request.strategy,
            env.plan.clone(),
            baseline_strategy,
            Some(champion_strategy_name),
            &request.command,
            &corpus,
            Some(&request.dense_model),
        ),
        metrics,
        baseline_metrics,
        champion_metrics,
        delta,
    })
}

pub fn run_latency_evaluation(
    request: &LatencyEvaluationRequest,
    ignore: Option<&Ignore>,
) -> Result<LatencyEvaluationReport> {
    let prepare_started = Instant::now();
    tracing::info!("→ loading dense model: {}", request.dense_model.model_id);
    let dense_for_load = std::sync::Arc::new(DenseReranker::load(request.dense_model.clone())?);
    let telemetry_for_load = std::sync::Arc::new(crate::system::Telemetry::new());
    let query_cache = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let corpus = load_search_corpus(&request.corpus_dir, ignore, request.verbose, Some(dense_for_load.as_ref()), &telemetry_for_load, None)?;
    let index = crate::search::Bm25Index::build(&corpus.documents);
    let queries = load_queries(&request.queries_path)?;
    let prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

    if queries.is_empty() {
        bail!("latency evaluation requires at least one query");
    }

    let env = crate::search::SearchEnvironment::new(
        &SearchRequest {
            strategy: request.strategy.clone(),
            query: String::new(),
            path: request.corpus_dir.clone(),
            limit: 10,
            shortlist: request.shortlist,
            dense_model: request.dense_model.clone(),
            verbose: request.verbose,
            retrievers: None,
            fusion: None,
            reranking: None,
            telemetry: telemetry_for_load.clone(),
            cache_dir: None,
            query_cache: Some(query_cache.clone()),
        },
        &corpus,
        &index,
        Some(dense_for_load.clone()),
    )?;

    let mut timings = Vec::with_capacity(queries.len());
    let mut queries_vec: Vec<_> = queries.values().collect();
    queries_vec.sort(); // Deterministic order
    
    let total_queries = if let Some(limit) = request.query_limit {
        limit.min(queries_vec.len())
    } else {
        queries_vec.len()
    };

    for query_text in queries_vec.iter().take(total_queries) {
        let started = Instant::now();
        let _ = env.search(query_text, 10, request.verbose)?;
        timings.push(started.elapsed().as_secs_f64() * 1000.0)
    }

    timings.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    let p50_ms = percentile(&timings, 0.50);
    let p90_ms = percentile(&timings, 0.90);
    let max_ms = timings.last().copied().unwrap_or(0.0);

    Ok(LatencyEvaluationReport {
        metadata: build_metadata(
            &request.strategy,
            env.plan.clone(),
            None,
            None,
            &request.command,
            &corpus,
            Some(&request.dense_model),
        ),
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

pub fn run_comparative_evaluation(
    request: &QualityEvaluationRequest,
    ignore: Option<&Ignore>,
) -> Result<ComparativeEvaluationReport> {
    let registry = crate::search::StrategyPresetRegistry::default_registry();
    let names = registry.names();
    let mut results = Vec::new();
    let mut metadata = Vec::new();

    let prepare_started = Instant::now();
    tracing::info!("→ loading dense model: {}", request.dense_model.model_id);
    let dense_for_load = std::sync::Arc::new(DenseReranker::load(request.dense_model.clone())?);
    let telemetry_for_load = std::sync::Arc::new(crate::system::Telemetry::new());
    let query_cache = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let corpus = load_search_corpus(&request.corpus_dir, ignore, request.verbose, Some(dense_for_load.as_ref()), &telemetry_for_load, None)?;
    let index = crate::search::Bm25Index::build(&corpus.documents);
    let queries_path = request
        .queries_path
        .clone()
        .unwrap_or_else(|| request.corpus_dir.join("test-queries.tsv"));
    let queries = load_queries(&queries_path)?;
    let qrels = load_qrels(&request.qrels_path)?;
    let _prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

    let total_strategies = names.len();
    for (idx, name) in names.iter().enumerate() {
        tracing::info!("→ evaluation strategy {}/{} : {}", idx + 1, total_strategies, name);
        let env = crate::search::SearchEnvironment::new(
            &SearchRequest {
                strategy: name.clone(),
                query: String::new(),
                path: request.corpus_dir.clone(),
                limit: request.shortlist,
                shortlist: request.shortlist,
                dense_model: request.dense_model.clone(),
                verbose: request.verbose,
                retrievers: None,
                fusion: None,
                reranking: None,
                telemetry: telemetry_for_load.clone(),
                cache_dir: None,
                query_cache: Some(query_cache.clone()),
            },
            &corpus,
            &index,
            Some(dense_for_load.clone()),
        )?;

        // Quality
        let (quality, telemetry) = evaluate_quality(
            &queries,
            &qrels,
            &env,
            request.verbose,
            request.query_limit,
        )?;

        // Latency
        let prepare_started = Instant::now();
        let prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

        let mut timings = Vec::with_capacity(queries.len());
        let mut queries_vec: Vec<_> = queries.values().collect();
        queries_vec.sort();

        let total_queries = if let Some(limit) = request.query_limit {
            limit.min(queries_vec.len())
        } else {
            queries_vec.len()
        };

        for query_text in queries_vec.iter().take(total_queries) {
            let started = Instant::now();
            let _ = env.search(query_text, 10, request.verbose)?;
            timings.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        timings.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

        let latency = LatencyMetrics {
            prepare_ms,
            p50_ms: percentile(&timings, 0.50),
            p90_ms: percentile(&timings, 0.90),
            max_ms: timings.last().copied().unwrap_or(0.0),
            target_ms: LATENCY_TARGET_MS,
            p50_over_target_ms: over_target_ms(percentile(&timings, 0.50)),
            p90_over_target_ms: over_target_ms(percentile(&timings, 0.90)),
            max_over_target_ms: over_target_ms(timings.last().copied().unwrap_or(0.0)),
        };

        results.push(StrategyComparison {
            strategy: name.clone(),
            quality,
            latency,
            telemetry: Some(telemetry),
        });

        metadata.push(build_metadata(
            &name,
            env.plan.clone(),
            None,
            None,
            &request.command,
            &corpus,
            Some(&request.dense_model),
        ));
    }

    Ok(ComparativeEvaluationReport { metadata, results })
}

pub fn render_comparative_report(report: &ComparativeEvaluationReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    writeln!(out, "\x1b[1mComparative Search Strategy Evaluation\x1b[0m").unwrap();
    writeln!(
        out,
        "──────────────────────────────────────────────────────────────────────────────────────────────"
    )
    .unwrap();
    writeln!(
        out,
        "{:<20} {:>10} {:>10} {:>10} {:>12} {:>15}",
        "Strategy", "nDCG@10", "MRR@10", "Recall@10", "p50 (ms)", "Cache Hits"
    )
    .unwrap();
    writeln!(
        out,
        "──────────────────────────────────────────────────────────────────────────────────────────────"
    )
    .unwrap();

    let ndcgs: Vec<f64> = report.results.iter().map(|r| r.quality.ndcg_at_10).collect();
    let mrrs: Vec<f64> = report.results.iter().map(|r| r.quality.mrr_at_10).collect();
    let recalls: Vec<f64> = report.results.iter().map(|r| r.quality.recall_at_10).collect();
    let latencies: Vec<f64> = report.results.iter().map(|r| r.latency.p50_ms).collect();

    for res in &report.results {
        let bar = render_bar(res.quality.ndcg_at_10, 10);
        let hits = if let Some(t) = &res.telemetry {
            format!("{:.0}/{:.0}/{:.0}%", t.heuristic_hit_rate * 100.0, t.blob_hit_rate * 100.0, t.embedding_hit_rate * 100.0)
        } else {
            "-".to_string()
        };

        let ndcg_c = get_color(res.quality.ndcg_at_10, &ndcgs, true);
        let mrr_c = get_color(res.quality.mrr_at_10, &mrrs, true);
        let recall_c = get_color(res.quality.recall_at_10, &recalls, true);
        let lat_c = get_color(res.latency.p50_ms, &latencies, false);

        writeln!(
            out,
            "{}{:<20}\x1b[0m {}{:>10.4}\x1b[0m {}{:>10.4}\x1b[0m {}{:>10.4}\x1b[0m {}{:>12.2}\x1b[0m {:>15}  {}",
            ndcg_c, res.strategy, // Use nDCG color for strategy name
            ndcg_c, res.quality.ndcg_at_10,
            mrr_c, res.quality.mrr_at_10,
            recall_c, res.quality.recall_at_10,
            lat_c, res.latency.p50_ms,
            hits,
            bar
        )
        .unwrap();
    }
    writeln!(
        out,
        "──────────────────────────────────────────────────────────────────────────────────────────────"
    )
    .unwrap();

    if let Some(meta) = report.metadata.first() {
        writeln!(out, "\n\x1b[1mEnvironment\x1b[0m").unwrap();
        writeln!(
            out,
            "  OS:       {} ({})",
            meta.hardware.os, meta.hardware.arch
        )
        .unwrap();
        writeln!(
            out,
            "  CPU:      {}",
            meta.hardware.cpu_brand.as_deref().unwrap_or("unknown")
        )
        .unwrap();
        writeln!(
            out,
            "  Corpus:   {} documents, {} bytes",
            meta.corpus_documents, meta.corpus_bytes
        )
        .unwrap();
        if let Some(sha) = &meta.git_sha {
            writeln!(out, "  Git SHA:  {}", sha).unwrap();
        }
    }

    out
}

fn get_color(value: f64, all_values: &[f64], higher_is_better: bool) -> &'static str {
    if all_values.len() < 3 {
        return "";
    }

    let mut sorted = all_values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min = sorted[0];
    let max = *sorted.last().unwrap();

    if (max - min).abs() < f64::EPSILON {
        return "";
    }

    let (best, worst) = if higher_is_better { (max, min) } else { (min, max) };

    if (value - best).abs() < f64::EPSILON {
        return "\x1b[1;32m"; // Bold Green
    }
    if (value - worst).abs() < f64::EPSILON {
        return "\x1b[1;31m"; // Bold Red
    }

    // Find the actual median value from the sorted list
    let len = sorted.len();
    let median = if len % 2 == 1 {
        sorted[len / 2]
    } else {
        // For even length, we pick one of the two middle ones to avoid "closest" ambiguity
        sorted[len / 2]
    };

    if (value - median).abs() < f64::EPSILON {
        "\x1b[1;33m" // Bold Yellow/Orange
    } else {
        ""
    }
}

fn render_bar(value: f64, width: usize) -> String {
    let filled = (value * width as f64).round() as usize;
    let mut bar = String::from("\x1b[32m");
    for _ in 0..filled {
        bar.push('█');
    }
    bar.push_str("\x1b[90m");
    for _ in filled..width {
        bar.push('░');
    }
    bar.push_str("\x1b[0m");
    bar
}

fn build_metadata(
    strategy: &str,
    plan: crate::search::SearchPlan,
    baseline_strategy: Option<String>,
    champion_strategy: Option<String>,
    command: &str,
    corpus: &LoadedCorpus,
    dense_model: Option<&DenseModelSpec>,
) -> EvaluationMetadata {
    EvaluationMetadata {
        strategy: strategy.to_string(),
        plan,
        baseline_strategy,
        champion_strategy,
        command: command.to_string(),
        git_sha: current_git_sha(),
        corpus_documents: corpus.documents.len(),
        corpus_bytes: corpus.total_bytes,
        segment_strategy: "structure-aware".to_string(),
        segment_count: corpus
            .documents
            .iter()
            .map(|document| document.segments().len())
            .sum(),
        hardware: detect_hardware_summary(),
        dense_model: dense_model.map(|spec| spec.model_id.clone()),
        dense_revision: dense_model.map(|spec| spec.revision.clone()),
        dense_max_length: dense_model.map(|spec| spec.max_length),
        shortlist: None,
    }
}

fn evaluate_quality(
    queries: &HashMap<String, String>,
    qrels: &HashMap<String, HashMap<String, u32>>,
    env: &crate::search::SearchEnvironment,
    verbose: u8,
    query_limit: Option<usize>,
) -> Result<(QualityMetrics, crate::search::SearchTelemetry)> {
    let mut ndcg_total = 0.0;
    let mut mrr_total = 0.0;
    let mut recall_total = 0.0;
    let mut counted_queries = 0_usize;

    let mut qrels_vec: Vec<_> = qrels.iter().collect();
    qrels_vec.sort_by_key(|(id, _)| *id);
    
    let total_queries = if let Some(limit) = query_limit {
        limit.min(qrels_vec.len())
    } else {
        qrels_vec.len()
    };

    for (i, (query_id, relevances)) in qrels_vec.iter().take(total_queries).enumerate() {
        if verbose > 0 && i > 0 && i % 50 == 0 {
            tracing::info!("    evaluated {}/{} queries...", i, total_queries);
        }
        let query_text = queries
            .get(*query_id)
            .with_context(|| format!("missing query text for qrels query-id '{query_id}'"))?;

        let response = env.search(query_text, 10, verbose)?;

        let ranked_ids: Vec<String> = response
            .results
            .iter()
            .map(|hit| {
                Path::new(&hit.path)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect();

        ndcg_total += ndcg_at_10(&ranked_ids, relevances);
        mrr_total += mrr_at_10(&ranked_ids, relevances);
        recall_total += recall_at_10(&ranked_ids, relevances);
        counted_queries += 1;
    }

    if counted_queries == 0 {
        bail!("quality evaluation requires at least one qrels row");
    }

    Ok((
        QualityMetrics {
            ndcg_at_10: ndcg_total / counted_queries as f64,
            mrr_at_10: mrr_total / counted_queries as f64,
            recall_at_10: recall_total / counted_queries as f64,
        },
        crate::search::SearchTelemetry {
            heuristic_hit_rate: env.telemetry.heuristic_hit_rate(),
            blob_hit_rate: env.telemetry.blob_hit_rate(),
            embedding_hit_rate: env.telemetry.embedding_hit_rate(),
        },
    ))
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

fn mrr_at_10(results: &[String], relevances: &HashMap<String, u32>) -> f64 {
    results
        .iter()
        .enumerate()
        .find(|(_, id)| relevances.get(*id).copied().unwrap_or(0) > 0)
        .map(|(index, _)| 1.0 / (index as f64 + 1.0))
        .unwrap_or(0.0)
}

fn recall_at_10(results: &[String], relevances: &HashMap<String, u32>) -> f64 {
    let relevant_total = relevances.values().filter(|score| **score > 0).count();
    if relevant_total == 0 {
        return 0.0;
    }

    let hits = results
        .iter()
        .filter(|id| relevances.get(*id).copied().unwrap_or(0) > 0)
        .count();

    hits as f64 / relevant_total as f64
}

fn ndcg_at_10(results: &[String], relevances: &HashMap<String, u32>) -> f64 {
    let dcg = results
        .iter()
        .enumerate()
        .map(|(index, id)| discounted_gain(index, relevances.get(id).copied().unwrap_or(0)))
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
