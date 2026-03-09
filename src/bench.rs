use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::dense::DenseModelSpec;
use crate::search::{LoadedCorpus, SearchRequest, load_materialized_corpus, run_search};
use crate::system::{HardwareSummary, current_git_sha, detect_hardware_summary};

const LATENCY_TARGET_MS: f64 = 200.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkMetadata {
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
pub struct QualityBenchmarkReport {
    pub metadata: BenchmarkMetadata,
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
pub struct LatencyBenchmarkReport {
    pub metadata: BenchmarkMetadata,
    pub latency_ms: LatencyMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComparativeBenchmarkReport {
    pub metadata: Vec<BenchmarkMetadata>,
    pub results: Vec<StrategyComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyComparison {
    pub strategy: String,
    pub quality: QualityMetrics,
    pub latency: LatencyMetrics,
}

#[derive(Debug, Clone)]
pub struct QualityBenchmarkRequest {
    pub strategy: String,
    pub baseline: Option<String>,
    pub command: String,
    pub corpus_dir: PathBuf,
    pub queries_path: Option<PathBuf>,
    pub qrels_path: PathBuf,
    pub shortlist: usize,
    pub dense_model: DenseModelSpec,
}

#[derive(Debug, Clone)]
pub struct LatencyBenchmarkRequest {
    pub strategy: String,
    pub command: String,
    pub corpus_dir: PathBuf,
    pub queries_path: PathBuf,
    pub shortlist: usize,
    pub dense_model: DenseModelSpec,
}

pub fn run_quality_benchmark(request: &QualityBenchmarkRequest) -> Result<QualityBenchmarkReport> {
    let prepare_started = Instant::now();
    let corpus = load_materialized_corpus(&request.corpus_dir)?;
    let queries_path = request
        .queries_path
        .clone()
        .unwrap_or_else(|| request.corpus_dir.join("test-queries.tsv"));
    let queries = load_queries(&queries_path)?;
    let qrels = load_qrels(&request.qrels_path)?;
    let _prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

    let registry = crate::search::StrategyPresetRegistry::default_registry();
    let plan = registry.resolve(&request.strategy)?;

    let metrics = evaluate_quality(
        &request.corpus_dir,
        &queries,
        &qrels,
        &request.strategy,
        request.shortlist,
        &request.dense_model,
    )?;

    let baseline_strategy = request
        .baseline
        .clone()
        .or_else(|| Some("bm25".to_string()));
    let baseline_metrics = match &baseline_strategy {
        Some(strategy) => Some(evaluate_quality(
            &request.corpus_dir,
            &queries,
            &qrels,
            strategy,
            request.shortlist,
            &request.dense_model,
        )?),
        None => None,
    };

    // Resolving "hybrid" gives us the champion plan
    let champion_plan = registry.resolve("hybrid")?;
    let champion_strategy_name = champion_plan.name.clone();
    let champion_strategy = Some(champion_strategy_name.clone());

    let champion_metrics = if champion_strategy_name != request.strategy {
        Some(evaluate_quality(
            &request.corpus_dir,
            &queries,
            &qrels,
            &champion_strategy_name,
            request.shortlist,
            &request.dense_model,
        )?)
    } else {
        Some(metrics.clone())
    };

    let delta = baseline_metrics
        .as_ref()
        .map(|baseline| quality_delta(&metrics, baseline));

    Ok(QualityBenchmarkReport {
        metadata: build_metadata(
            &request.strategy,
            plan,
            baseline_strategy,
            champion_strategy,
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

pub fn run_latency_benchmark(request: &LatencyBenchmarkRequest) -> Result<LatencyBenchmarkReport> {
    let prepare_started = Instant::now();
    let corpus = load_materialized_corpus(&request.corpus_dir)?;
    let queries = load_queries(&request.queries_path)?;
    let prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

    if queries.is_empty() {
        bail!("latency benchmark requires at least one query");
    }

    let registry = crate::search::StrategyPresetRegistry::default_registry();
    let plan = registry.resolve(&request.strategy)?;

    let mut timings = Vec::with_capacity(queries.len());
    for query_text in queries.values() {
        let started = Instant::now();
        let _ = run_search(&SearchRequest {
            strategy: request.strategy.clone(),
            query: query_text.clone(),
            path: request.corpus_dir.clone(),
            limit: 10,
            shortlist: request.shortlist,
            dense_model: request.dense_model.clone(),
        })?;
        timings.push(started.elapsed().as_secs_f64() * 1000.0);
    }

    timings.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    let p50_ms = percentile(&timings, 0.50);
    let p90_ms = percentile(&timings, 0.90);
    let max_ms = timings.last().copied().unwrap_or(0.0);

    Ok(LatencyBenchmarkReport {
        metadata: build_metadata(
            &request.strategy,
            plan,
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

pub fn run_comparative_benchmark(
    request: &QualityBenchmarkRequest,
) -> Result<ComparativeBenchmarkReport> {
    let registry = crate::search::StrategyPresetRegistry::default_registry();
    let names = registry.names();
    let mut results = Vec::new();
    let mut metadata = Vec::new();

    // Prepare shared state for latency
    let corpus = load_materialized_corpus(&request.corpus_dir)?;
    let queries_path = request
        .queries_path
        .clone()
        .unwrap_or_else(|| request.corpus_dir.join("test-queries.tsv"));
    let queries = load_queries(&queries_path)?;
    let qrels = load_qrels(&request.qrels_path)?;

    for name in names {
        let plan = registry.resolve(&name)?;
        
        // Quality
        let quality = evaluate_quality(
            &request.corpus_dir,
            &queries,
            &qrels,
            &name,
            request.shortlist,
            &request.dense_model,
        )?;

        // Latency
        let prepare_started = Instant::now();
        let prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;
        
        let mut timings = Vec::with_capacity(queries.len());
        for query_text in queries.values() {
            let started = Instant::now();
            let _ = run_search(&SearchRequest {
                strategy: name.clone(),
                query: query_text.clone(),
                path: request.corpus_dir.clone(),
                limit: 10,
                shortlist: request.shortlist,
                dense_model: request.dense_model.clone(),
            })?;
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
        });

        metadata.push(build_metadata(
            &name,
            plan,
            None,
            None,
            &request.command,
            &corpus,
            Some(&request.dense_model),
        ));
    }

    Ok(ComparativeBenchmarkReport { metadata, results })
}

pub fn render_comparative_report(report: &ComparativeBenchmarkReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    writeln!(out, "\x1b[1mComparative Search Strategy Benchmark\x1b[0m").unwrap();
    writeln!(out, "────────────────────────────────────────────────────────────────────────────────").unwrap();
    writeln!(out, "{:<20} {:>10} {:>10} {:>10} {:>12}", "Strategy", "nDCG@10", "MRR@10", "Recall@10", "p50 (ms)").unwrap();
    writeln!(out, "────────────────────────────────────────────────────────────────────────────────").unwrap();

    for res in &report.results {
        let bar = render_bar(res.quality.ndcg_at_10, 10);
        writeln!(
            out,
            "{:<20} {:>10.4} {:>10.4} {:>10.4} {:>12.2}  {}",
            res.strategy,
            res.quality.ndcg_at_10,
            res.quality.mrr_at_10,
            res.quality.recall_at_10,
            res.latency.p50_ms,
            bar
        ).unwrap();
    }
    writeln!(out, "────────────────────────────────────────────────────────────────────────────────").unwrap();
    
    if let Some(meta) = report.metadata.first() {
        writeln!(out, "\n\x1b[1mEnvironment\x1b[0m").unwrap();
        writeln!(out, "  OS:       {} ({})", meta.hardware.os, meta.hardware.arch).unwrap();
        writeln!(out, "  CPU:      {}", meta.hardware.cpu_brand.as_deref().unwrap_or("unknown")).unwrap();
        writeln!(out, "  Corpus:   {} documents, {} bytes", meta.corpus_documents, meta.corpus_bytes).unwrap();
        if let Some(sha) = &meta.git_sha {
            writeln!(out, "  Git SHA:  {}", sha).unwrap();
        }
    }

    out
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
) -> BenchmarkMetadata {
    BenchmarkMetadata {
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
    corpus_dir: &Path,
    queries: &HashMap<String, String>,
    qrels: &HashMap<String, HashMap<String, u32>>,
    strategy: &str,
    shortlist: usize,
    dense_model: &DenseModelSpec,
) -> Result<QualityMetrics> {
    let mut ndcg_total = 0.0;
    let mut mrr_total = 0.0;
    let mut recall_total = 0.0;
    let mut counted_queries = 0_usize;

    for (query_id, relevances) in qrels {
        let query_text = queries
            .get(query_id)
            .with_context(|| format!("missing query text for qrels query-id '{query_id}'"))?;

        let response = run_search(&SearchRequest {
            strategy: strategy.to_string(),
            query: query_text.clone(),
            path: corpus_dir.to_path_buf(),
            limit: 10,
            shortlist,
            dense_model: dense_model.clone(),
        })?;

        // Map SearchHit to something we can use for metrics
        // We need the ID, which is not currently in SearchHit.
        // Wait, SearchHit.path is used as ID in some places or we need a way to get ID.
        // In materialized corpus, ID is the file stem.

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
