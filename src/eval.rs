use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, anyhow, bail};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};

use crate::cache::resolve_compatible_cache_path;
use crate::config::Ignore;
use crate::dense::{DenseModelSpec, DenseReranker};
use crate::search::{LoadedCorpus, SearchEngine, SearchRequest, load_search_corpus};
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
    pub artifacts: usize,
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
    let source_dir = resolve_compatible_cache_path(source_dir);
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

    let mut artifacts = 0;
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
        artifacts += 1;
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
        artifacts,
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
    let path = resolve_compatible_cache_path(path);
    let file = fs::File::open(&path).with_context(|| format!("open archive {}", path.display()))?;
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
    let path = resolve_compatible_cache_path(path);
    let file = fs::File::open(&path).with_context(|| format!("open qrels {}", path.display()))?;
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
pub struct ReactorMetrics {
    pub shortlist_compression: f64,
    pub signal_gain: f64,
    pub emission_fidelity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityEvaluationReport {
    pub metadata: EvaluationMetadata,
    pub metrics: QualityMetrics,
    pub reactor_metrics: Option<ReactorMetrics>,
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
    pub hits: Vec<StrategyComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgenticFixtureSet {
    pub tasks: Vec<AgenticTaskFixture>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgenticTaskFixture {
    pub id: String,
    pub turns: Vec<AgenticTurnFixture>,
    #[serde(default)]
    pub expected_final_documents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgenticTurnFixture {
    pub query: String,
    #[serde(default)]
    pub expected_documents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgenticEvaluationMetrics {
    pub task_success_rate: f64,
    pub average_turn_recall: f64,
    pub average_final_recall: f64,
    pub average_turns: f64,
    pub average_prune_actions: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgenticTurnEvaluation {
    pub turn_id: String,
    pub query: String,
    pub expected_documents: Vec<String>,
    pub hit_documents: Vec<String>,
    pub recall_at_10: f64,
    pub prune_actions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgenticTaskEvaluation {
    pub task_id: String,
    pub success: bool,
    pub turns_executed: usize,
    pub prune_actions: usize,
    pub latency_ms: f64,
    pub final_documents: Vec<String>,
    pub expected_final_documents: Vec<String>,
    pub final_recall_at_10: f64,
    pub trace: crate::search::SearchTrace,
    pub turns: Vec<AgenticTurnEvaluation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgenticComparisonMetrics {
    pub task_success_rate: f64,
    pub average_final_recall: f64,
    pub average_turns: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgenticComparisonRun {
    pub strategy: String,
    pub mode: String,
    pub metrics: AgenticComparisonMetrics,
    pub latency_ms: LatencyMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgenticTaskComparison {
    pub task_id: String,
    pub collapsed_query: String,
    pub expected_final_documents: Vec<String>,
    pub agentic_final_documents: Vec<String>,
    pub baseline_final_documents: Vec<String>,
    pub agentic_success: bool,
    pub baseline_success: bool,
    pub agentic_final_recall_at_10: f64,
    pub baseline_final_recall_at_10: f64,
    pub agentic_latency_ms: f64,
    pub baseline_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgenticComparisonDelta {
    pub task_success_rate: f64,
    pub average_final_recall: f64,
    pub average_turns: f64,
    pub p50_latency_ms: f64,
    pub p90_latency_ms: f64,
    pub max_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgenticComparisonReport {
    pub baseline_strategy: String,
    pub baseline_query_mode: String,
    pub agentic: AgenticComparisonRun,
    pub baseline: AgenticComparisonRun,
    pub delta: AgenticComparisonDelta,
    pub tasks: Vec<AgenticTaskComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgenticEvaluationReport {
    pub metadata: EvaluationMetadata,
    pub metrics: AgenticEvaluationMetrics,
    pub tasks: Vec<AgenticTaskEvaluation>,
    pub comparison: AgenticComparisonReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyComparison {
    pub strategy: String,
    pub expansion: String,
    pub quality: QualityMetrics,
    pub latency: LatencyMetrics,
    pub telemetry: Option<crate::search::SearchTelemetry>,
    pub reactor_metrics: Option<ReactorMetrics>,
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
    pub prompts: Option<crate::config::PromptsConfig>,
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

#[derive(Debug, Clone)]
pub struct AgenticEvaluationRequest {
    pub strategy: String,
    pub baseline_strategy: Option<String>,
    pub command: String,
    pub corpus_dir: PathBuf,
    pub fixtures_path: PathBuf,
    pub shortlist: usize,
    pub dense_model: DenseModelSpec,
    pub retained_artifact_limit: usize,
    pub verbose: u8,
    pub prompts: Option<crate::config::PromptsConfig>,
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
    let queries_path = request
        .queries_path
        .clone()
        .unwrap_or_else(|| request.corpus_dir.join("test-queries.tsv"));
    let corpus = filter_evaluation_helper_documents(
        load_search_corpus(
            &request.corpus_dir,
            ignore,
            request.verbose,
            Some(dense_for_load.as_ref()),
            &telemetry_for_load,
            &[],
            None,
        )?,
        &[queries_path.clone(), request.qrels_path.clone()],
    );
    let index = crate::search::Bm25Index::build(&corpus.artifacts);
    let queries = load_queries(&queries_path)?;
    let qrels = load_qrels(&request.qrels_path)?;
    let _prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

    let registry = crate::search::StrategyPresetRegistry::default_registry();
    let champion_plan = registry.resolve("hybrid")?;
    let champion_strategy_name = champion_plan.name.clone();

    let mut search_req =
        SearchRequest::new(&request.strategy, String::new(), request.corpus_dir.clone());
    search_req.limit = request.shortlist;
    search_req.shortlist = request.shortlist;
    search_req.dense_model = request.dense_model.clone();
    search_req.verbose = request.verbose;
    search_req.telemetry = telemetry_for_load.clone();
    search_req.query_cache = Some(query_cache.clone());
    search_req.prompts = request.prompts.clone();

    let env_plan = registry.resolve(&request.strategy)?;
    let env_llm = crate::search::SearchServiceBuilder::load_llm_reranker(&env_plan, &search_req)?;

    let env = crate::search::SearchEnvironment::new(
        &search_req,
        &corpus,
        &index,
        Some(dense_for_load.clone()),
        env_llm,
    )?;

    let (metrics, _telemetry) = evaluate_quality(
        &queries,
        &qrels,
        &env,
        request.shortlist,
        request.verbose,
        request.query_limit,
    )?;

    let baseline_strategy = request
        .baseline
        .clone()
        .or_else(|| Some("bm25".to_string()));
    let baseline_metrics = match &baseline_strategy {
        Some(strategy) => {
            let mut baseline_req =
                SearchRequest::new(strategy, String::new(), request.corpus_dir.clone());
            baseline_req.limit = request.shortlist;
            baseline_req.shortlist = request.shortlist;
            baseline_req.dense_model = request.dense_model.clone();
            baseline_req.verbose = request.verbose;
            baseline_req.telemetry = telemetry_for_load.clone();
            baseline_req.query_cache = Some(query_cache.clone());
            baseline_req.prompts = request.prompts.clone();

            let baseline_plan = registry.resolve(strategy)?;
            let baseline_llm = crate::search::SearchServiceBuilder::load_llm_reranker(
                &baseline_plan,
                &baseline_req,
            )?;

            let baseline_env = crate::search::SearchEnvironment::new(
                &baseline_req,
                &corpus,
                &index,
                Some(dense_for_load.clone()),
                baseline_llm,
            )?;
            let (m, _) = evaluate_quality(
                &queries,
                &qrels,
                &baseline_env,
                request.shortlist,
                request.verbose,
                request.query_limit,
            )?;
            Some(m)
        }
        None => None,
    };

    let champion_metrics = if champion_strategy_name != request.strategy {
        let mut champion_req = SearchRequest::new(
            &champion_strategy_name,
            String::new(),
            request.corpus_dir.clone(),
        );
        champion_req.limit = request.shortlist;
        champion_req.shortlist = request.shortlist;
        champion_req.dense_model = request.dense_model.clone();
        champion_req.verbose = request.verbose;
        champion_req.telemetry = telemetry_for_load.clone();
        champion_req.query_cache = Some(query_cache.clone());
        champion_req.prompts = request.prompts.clone();

        let champion_llm =
            crate::search::SearchServiceBuilder::load_llm_reranker(&champion_plan, &champion_req)?;

        let champion_env = crate::search::SearchEnvironment::new(
            &champion_req,
            &corpus,
            &index,
            Some(dense_for_load.clone()),
            champion_llm,
        )?;
        let (m, _) = evaluate_quality(
            &queries,
            &qrels,
            &champion_env,
            request.shortlist,
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

    let signal_gain = baseline_metrics
        .as_ref()
        .map(|baseline| metrics.ndcg_at_10 - baseline.ndcg_at_10)
        .unwrap_or(0.0);

    Ok(QualityEvaluationReport {
        metadata: build_metadata(
            &request.strategy,
            env.ir.plan.clone(),
            baseline_strategy,
            Some(champion_strategy_name),
            &request.command,
            &corpus,
            Some(&request.dense_model),
        ),
        metrics,
        reactor_metrics: Some(ReactorMetrics {
            shortlist_compression: request.shortlist as f64 / 10.0,
            signal_gain,
            emission_fidelity: 1.0,
        }),
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
    let corpus = filter_evaluation_helper_documents(
        load_search_corpus(
            &request.corpus_dir,
            ignore,
            request.verbose,
            Some(dense_for_load.as_ref()),
            &telemetry_for_load,
            &[],
            None,
        )?,
        std::slice::from_ref(&request.queries_path),
    );
    let index = crate::search::Bm25Index::build(&corpus.artifacts);
    let queries = load_queries(&request.queries_path)?;
    let prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

    if queries.is_empty() {
        bail!("latency evaluation requires at least one query");
    }

    let mut latency_req =
        SearchRequest::new(&request.strategy, String::new(), request.corpus_dir.clone());
    latency_req.shortlist = request.shortlist;
    latency_req.dense_model = request.dense_model.clone();
    latency_req.verbose = request.verbose;
    latency_req.telemetry = telemetry_for_load.clone();
    latency_req.query_cache = Some(query_cache.clone());

    let registry = crate::search::StrategyPresetRegistry::default_registry();
    let latency_plan = registry.resolve(&request.strategy)?;
    let latency_llm =
        crate::search::SearchServiceBuilder::load_llm_reranker(&latency_plan, &latency_req)?;

    let env = crate::search::SearchEnvironment::new(
        &latency_req,
        &corpus,
        &index,
        Some(dense_for_load.clone()),
        latency_llm,
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
        let mut turn_req = SearchRequest::new(
            &request.strategy,
            query_text.to_string(),
            request.corpus_dir.clone(),
        );
        turn_req.shortlist = request.shortlist;
        turn_req.verbose = request.verbose;
        turn_req.telemetry = env.telemetry.clone();

        let _ = env.search(&turn_req)?;
        timings.push(started.elapsed().as_secs_f64() * 1000.0)
    }

    timings.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    let p50_ms = percentile(&timings, 0.50);
    let p90_ms = percentile(&timings, 0.90);
    let max_ms = timings.last().copied().unwrap_or(0.0);

    Ok(LatencyEvaluationReport {
        metadata: build_metadata(
            &request.strategy,
            env.ir.plan.clone(),
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

pub fn run_agentic_evaluation(
    request: &AgenticEvaluationRequest,
    ignore: Option<&Ignore>,
) -> Result<AgenticEvaluationReport> {
    let fixtures = load_agentic_fixture_set(&request.fixtures_path)?;
    if fixtures.tasks.is_empty() {
        bail!("agentic evaluation requires at least one task fixture");
    }

    let registry = crate::search::StrategyPresetRegistry::default_registry();
    let plan = registry.resolve(&request.strategy)?;
    let baseline_strategy = request
        .baseline_strategy
        .clone()
        .unwrap_or_else(|| "hybrid".to_string());

    let metadata_telemetry = std::sync::Arc::new(crate::system::Telemetry::new());
    let corpus = filter_evaluation_helper_documents(
        load_search_corpus(
            &request.corpus_dir,
            ignore,
            request.verbose,
            None,
            &metadata_telemetry,
            &[],
            None,
        )?,
        std::slice::from_ref(&request.fixtures_path),
    );

    let mut config = crate::config::Config::default();
    config.search.strategy = request.strategy.clone();
    config.search.shortlist = request.shortlist;
    config.embedding.model_id = request.dense_model.model_id.clone();
    config.embedding.model_revision = request.dense_model.revision.clone();
    config.embedding.max_length = request.dense_model.max_length;
    if let Some(prompts) = &request.prompts {
        config.prompts = prompts.clone();
    }

    let query_cache = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let telemetry = std::sync::Arc::new(crate::system::Telemetry::new());
    let builder = crate::facade::Sift::builder()
        .with_config(config)
        .with_telemetry(telemetry)
        .with_query_cache(query_cache);
    let engine = match ignore {
        Some(_) => builder.with_ignore(crate::config::Ignore::load()).build(),
        None => builder.without_ignore().build(),
    };

    let mut task_results = Vec::new();
    let mut successful_tasks = 0_usize;
    let mut total_turns = 0_usize;
    let mut total_prune_actions = 0_usize;
    let mut recall_sum = 0.0;
    let mut recall_count = 0_usize;
    let mut final_recall_sum = 0.0;
    let mut agentic_timings = Vec::new();
    let mut baseline_task_successes = 0_usize;
    let mut baseline_final_recall_sum = 0.0;
    let mut baseline_timings = Vec::new();
    let mut task_comparisons = Vec::new();

    for task in &fixtures.tasks {
        if task.turns.is_empty() {
            bail!("agentic task '{}' requires at least one turn", task.id);
        }

        let turns = task
            .turns
            .iter()
            .enumerate()
            .map(|(index, turn)| {
                crate::search::SearchTurnRequest::new(&request.corpus_dir, turn.query.clone())
                    .with_session_id(task.id.clone())
                    .with_turn_id(format!("turn-{}", index + 1))
                    .with_sequence(index + 1)
                    .with_limit(10)
                    .with_shortlist(request.shortlist)
                    .with_emission_mode(crate::search::SearchEmissionMode::Protocol)
            })
            .collect();

        let agentic_started = Instant::now();
        let response = engine.search_controller(
            crate::search::SearchControllerRequest::new(plan.clone(), turns)
                .with_session_id(task.id.clone())
                .with_retained_artifact_limit(request.retained_artifact_limit),
        )?;
        let agentic_latency_ms = agentic_started.elapsed().as_secs_f64() * 1000.0;
        if response.turns.len() != task.turns.len() {
            bail!(
                "agentic task '{}' executed {} turn(s), expected {}",
                task.id,
                response.turns.len(),
                task.turns.len()
            );
        }

        let mut turn_results = Vec::new();
        let mut task_prune_actions = 0_usize;

        for (turn_response, fixture_turn) in response.turns.iter().zip(task.turns.iter()) {
            let trace_turn = turn_response.trace.turns.first().ok_or_else(|| {
                anyhow!("missing trace for turn '{}'", turn_response.turn.turn_id)
            })?;
            let prune_actions = trace_turn
                .decisions
                .iter()
                .filter(|decision| decision.action == crate::search::SearchControllerAction::Prune)
                .count();
            task_prune_actions += prune_actions;

            let hit_documents = document_ids_from_emission(&turn_response.emission);

            let recall_at_10 = if fixture_turn.expected_documents.is_empty() {
                1.0
            } else {
                recall_against_expected(&hit_documents, &fixture_turn.expected_documents)
            };
            if !fixture_turn.expected_documents.is_empty() {
                recall_sum += recall_at_10;
                recall_count += 1;
            }

            turn_results.push(AgenticTurnEvaluation {
                turn_id: turn_response.turn.turn_id.clone(),
                query: fixture_turn.query.clone(),
                expected_documents: fixture_turn.expected_documents.clone(),
                hit_documents,
                recall_at_10,
                prune_actions,
            });
        }

        let final_documents: Vec<_> = response
            .state
            .retained_artifacts
            .iter()
            .map(|evidence| search_path_to_document_id(&evidence.path))
            .collect();
        let success =
            expected_documents_satisfied(&task.expected_final_documents, &final_documents);
        let final_recall_at_10 =
            recall_against_expected(&final_documents, &task.expected_final_documents);
        if success {
            successful_tasks += 1;
        }
        final_recall_sum += final_recall_at_10;
        agentic_timings.push(agentic_latency_ms);

        let collapsed_query = collapse_agentic_task_query(task);
        let baseline_started = Instant::now();
        let baseline_response = engine.search_turn(
            crate::search::SearchTurnRequest::new(&request.corpus_dir, collapsed_query.clone())
                .with_session_id(format!("{}-baseline", task.id))
                .with_turn_id("baseline-turn")
                .with_strategy(baseline_strategy.clone())
                .with_limit(10)
                .with_shortlist(request.shortlist)
                .with_verbose(request.verbose)
                .with_emission_mode(crate::search::SearchEmissionMode::Protocol),
        )?;
        let baseline_latency_ms = baseline_started.elapsed().as_secs_f64() * 1000.0;
        let baseline_final_documents = document_ids_from_emission(&baseline_response.emission);
        let baseline_success =
            expected_documents_satisfied(&task.expected_final_documents, &baseline_final_documents);
        let baseline_final_recall_at_10 =
            recall_against_expected(&baseline_final_documents, &task.expected_final_documents);
        if baseline_success {
            baseline_task_successes += 1;
        }
        baseline_final_recall_sum += baseline_final_recall_at_10;
        baseline_timings.push(baseline_latency_ms);

        total_turns += response.turns.len();
        total_prune_actions += task_prune_actions;
        task_results.push(AgenticTaskEvaluation {
            task_id: task.id.clone(),
            success,
            turns_executed: response.turns.len(),
            prune_actions: task_prune_actions,
            latency_ms: agentic_latency_ms,
            final_documents,
            expected_final_documents: task.expected_final_documents.clone(),
            final_recall_at_10,
            trace: response.trace.clone(),
            turns: turn_results,
        });
        task_comparisons.push(AgenticTaskComparison {
            task_id: task.id.clone(),
            collapsed_query,
            expected_final_documents: task.expected_final_documents.clone(),
            agentic_final_documents: task_results
                .last()
                .map(|task| task.final_documents.clone())
                .unwrap_or_default(),
            baseline_final_documents,
            agentic_success: success,
            baseline_success,
            agentic_final_recall_at_10: final_recall_at_10,
            baseline_final_recall_at_10,
            agentic_latency_ms,
            baseline_latency_ms,
        });
    }

    let mut metadata = build_metadata(
        &request.strategy,
        plan,
        Some(baseline_strategy.clone()),
        None,
        &request.command,
        &corpus,
        Some(&request.dense_model),
    );
    metadata.shortlist = Some(request.shortlist);

    let task_count = fixtures.tasks.len() as f64;
    let average_turn_recall = if recall_count == 0 {
        1.0
    } else {
        recall_sum / recall_count as f64
    };
    let average_final_recall = final_recall_sum / task_count;
    let agentic_run = AgenticComparisonRun {
        strategy: request.strategy.clone(),
        mode: "planned-controller".to_string(),
        metrics: AgenticComparisonMetrics {
            task_success_rate: successful_tasks as f64 / task_count,
            average_final_recall,
            average_turns: total_turns as f64 / task_count,
        },
        latency_ms: summarize_latencies(&agentic_timings),
    };
    let baseline_run = AgenticComparisonRun {
        strategy: baseline_strategy.clone(),
        mode: "collapsed-single-turn".to_string(),
        metrics: AgenticComparisonMetrics {
            task_success_rate: baseline_task_successes as f64 / task_count,
            average_final_recall: baseline_final_recall_sum / task_count,
            average_turns: 1.0,
        },
        latency_ms: summarize_latencies(&baseline_timings),
    };

    Ok(AgenticEvaluationReport {
        metadata,
        metrics: AgenticEvaluationMetrics {
            task_success_rate: agentic_run.metrics.task_success_rate,
            average_turn_recall,
            average_final_recall,
            average_turns: agentic_run.metrics.average_turns,
            average_prune_actions: total_prune_actions as f64 / task_count,
        },
        tasks: task_results,
        comparison: AgenticComparisonReport {
            baseline_strategy,
            baseline_query_mode: "concatenate-planned-turn-queries".to_string(),
            delta: AgenticComparisonDelta {
                task_success_rate: agentic_run.metrics.task_success_rate
                    - baseline_run.metrics.task_success_rate,
                average_final_recall: agentic_run.metrics.average_final_recall
                    - baseline_run.metrics.average_final_recall,
                average_turns: agentic_run.metrics.average_turns
                    - baseline_run.metrics.average_turns,
                p50_latency_ms: agentic_run.latency_ms.p50_ms - baseline_run.latency_ms.p50_ms,
                p90_latency_ms: agentic_run.latency_ms.p90_ms - baseline_run.latency_ms.p90_ms,
                max_latency_ms: agentic_run.latency_ms.max_ms - baseline_run.latency_ms.max_ms,
            },
            agentic: agentic_run,
            baseline: baseline_run,
            tasks: task_comparisons,
        },
    })
}

pub fn run_comparative_evaluation(
    request: &QualityEvaluationRequest,
    ignore: Option<&Ignore>,
) -> Result<ComparativeEvaluationReport> {
    let registry = crate::search::StrategyPresetRegistry::default_registry();
    let names = registry.names();
    let mut hits = Vec::new();
    let mut metadata = Vec::new();

    let prepare_started = Instant::now();
    tracing::info!("→ loading dense model: {}", request.dense_model.model_id);
    let dense_for_load = std::sync::Arc::new(DenseReranker::load(request.dense_model.clone())?);
    let telemetry_for_load = std::sync::Arc::new(crate::system::Telemetry::new());
    let query_cache = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let queries_path = request
        .queries_path
        .clone()
        .unwrap_or_else(|| request.corpus_dir.join("test-queries.tsv"));
    let corpus = filter_evaluation_helper_documents(
        load_search_corpus(
            &request.corpus_dir,
            ignore,
            request.verbose,
            Some(dense_for_load.as_ref()),
            &telemetry_for_load,
            &[],
            None,
        )?,
        &[queries_path.clone(), request.qrels_path.clone()],
    );
    let index = crate::search::Bm25Index::build(&corpus.artifacts);
    let queries = load_queries(&queries_path)?;
    let qrels = load_qrels(&request.qrels_path)?;
    let _prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

    let total_strategies = names.len();
    for (idx, name) in names.iter().enumerate() {
        tracing::info!(
            "→ evaluation strategy {}/{} : {}",
            idx + 1,
            total_strategies,
            name
        );
        let mut comp_req = SearchRequest::new(name, String::new(), request.corpus_dir.clone());
        comp_req.limit = request.shortlist;
        comp_req.shortlist = request.shortlist;
        comp_req.dense_model = request.dense_model.clone();
        comp_req.verbose = request.verbose;
        comp_req.telemetry = telemetry_for_load.clone();
        comp_req.query_cache = Some(query_cache.clone());
        comp_req.prompts = request.prompts.clone();

        let comp_plan = registry.resolve(name)?;
        let comp_llm =
            match crate::search::SearchServiceBuilder::load_llm_reranker(&comp_plan, &comp_req) {
                Ok(llm) => llm,
                Err(err) if is_gated_model_error(&err) => {
                    tracing::warn!(
                        "skipping evaluation strategy {} due to gated model access: {:#}",
                        name,
                        err
                    );
                    continue;
                }
                Err(err) => return Err(err),
            };

        let env = crate::search::SearchEnvironment::new(
            &comp_req,
            &corpus,
            &index,
            Some(dense_for_load.clone()),
            comp_llm,
        )?;

        // Quality
        let (quality, telemetry) = evaluate_quality(
            &queries,
            &qrels,
            &env,
            request.shortlist,
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
            let mut turn_req =
                SearchRequest::new(name, query_text.to_string(), request.corpus_dir.clone());
            turn_req.shortlist = request.shortlist;
            turn_req.verbose = request.verbose;
            turn_req.telemetry = env.telemetry.clone();

            let _ = env.search(&turn_req)?;
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

        hits.push(StrategyComparison {
            strategy: name.clone(),
            expansion: format!("{:?}", env.ir.plan.query_expansion),
            quality,
            latency,
            telemetry: Some(telemetry),
            reactor_metrics: Some(ReactorMetrics {
                shortlist_compression: request.shortlist as f64 / 10.0, // Placeholder ratio
                signal_gain: 0.0,       // Calculated later against baseline
                emission_fidelity: 1.0, // Placeholder
            }),
        });

        metadata.push(build_metadata(
            name,
            env.ir.plan.clone(),
            None,
            None,
            &request.command,
            &corpus,
            Some(&request.dense_model),
        ));
    }

    // Calculate signal gain against bm25 baseline
    let baseline_ndcg = hits
        .iter()
        .find(|r| r.strategy == "bm25")
        .map(|r| r.quality.ndcg_at_10)
        .unwrap_or(0.0);

    for res in &mut hits {
        if let Some(rm) = &mut res.reactor_metrics {
            rm.signal_gain = res.quality.ndcg_at_10 - baseline_ndcg;
        }
    }

    Ok(ComparativeEvaluationReport { metadata, hits })
}

fn is_gated_model_error(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        let message = cause.to_string().to_ascii_lowercase();
        message.contains("http status: 401")
            || message.contains("http status: 403")
            || (message.contains("huggingface") && message.contains("unauthorized"))
            || (message.contains("huggingface") && message.contains("forbidden"))
    })
}

pub fn render_comparative_report(report: &ComparativeEvaluationReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    writeln!(out, "\x1b[1mComparative Search Strategy Evaluation\x1b[0m").unwrap();
    writeln!(
        out,
        "────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────"
    )
    .unwrap();
    writeln!(
        out,
        "{:<25} {:<12} {:>10} {:>10} {:>10} {:>10} {:>10} {:>12} {:>15}",
        "Strategy",
        "Expansion",
        "nDCG@10",
        "MRR@10",
        "Recall@10",
        "S-Compress",
        "S-Gain",
        "p50 (ms)",
        "Cache Hits"
    )
    .unwrap();
    writeln!(
        out,
        "────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────"
    )
    .unwrap();

    let ndcgs: Vec<f64> = report.hits.iter().map(|r| r.quality.ndcg_at_10).collect();
    let mrrs: Vec<f64> = report.hits.iter().map(|r| r.quality.mrr_at_10).collect();
    let recalls: Vec<f64> = report.hits.iter().map(|r| r.quality.recall_at_10).collect();
    let latencies: Vec<f64> = report.hits.iter().map(|r| r.latency.p50_ms).collect();
    let gains: Vec<f64> = report
        .hits
        .iter()
        .map(|r| {
            r.reactor_metrics
                .as_ref()
                .map(|m| m.signal_gain)
                .unwrap_or(0.0)
        })
        .collect();

    for res in &report.hits {
        let bar = render_bar(res.quality.ndcg_at_10, 10);
        let hits = if let Some(t) = &res.telemetry {
            format!(
                "{}/{}/{} hits",
                t.heuristic_hits, t.blob_hits, t.embedding_hits
            )
        } else {
            "-".to_string()
        };

        let ndcg_c = get_color(res.quality.ndcg_at_10, &ndcgs, true);
        let mrr_c = get_color(res.quality.mrr_at_10, &mrrs, true);
        let recall_c = get_color(res.quality.recall_at_10, &recalls, true);
        let lat_c = get_color(res.latency.p50_ms, &latencies, false);

        let signal_gain = res
            .reactor_metrics
            .as_ref()
            .map(|m| m.signal_gain)
            .unwrap_or(0.0);
        let s_compress = res
            .reactor_metrics
            .as_ref()
            .map(|m| m.shortlist_compression)
            .unwrap_or(0.0);
        let gain_c = get_color(signal_gain, &gains, true);

        writeln!(
            out,
            "{}{:<25}\x1b[0m {:<12} {}{:>10.4}\x1b[0m {}{:>10.4}\x1b[0m {}{:>10.4}\x1b[0m {:>10.2} {}{:>10.4}\x1b[0m {}{:>12.2}\x1b[0m {:>15}  {}",
            ndcg_c, res.strategy, // Use nDCG color for strategy name
            res.expansion,
            ndcg_c, res.quality.ndcg_at_10,
            mrr_c, res.quality.mrr_at_10,
            recall_c, res.quality.recall_at_10,
            s_compress,
            gain_c, signal_gain,
            lat_c, res.latency.p50_ms,
            hits,
            bar
        )
        .unwrap();
    }
    writeln!(
        out,
        "────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────"
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
        writeln!(out, "  CPU:      {}", meta.hardware.cpu_brand).unwrap();
        writeln!(
            out,
            "  Corpus:   {} artifacts, {} bytes",
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

    let (best, worst) = if higher_is_better {
        (max, min)
    } else {
        (min, max)
    };

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
        git_sha: Some(current_git_sha()),
        corpus_documents: corpus.artifacts.len(),
        corpus_bytes: corpus.total_bytes,
        segment_strategy: "structure-aware".to_string(),
        segment_count: corpus
            .artifacts
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
    shortlist: usize,
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

        let mut turn_req =
            SearchRequest::new(&env.ir.plan.name, query_text.to_string(), PathBuf::new());
        turn_req.shortlist = shortlist;
        turn_req.limit = 10;
        turn_req.verbose = verbose;
        turn_req.telemetry = env.telemetry.clone();

        let response = env.search(&turn_req)?;

        let ranked_ids: Vec<String> = response
            .hits
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
            heuristic_hits: env
                .telemetry
                .heuristic_hits
                .load(std::sync::atomic::Ordering::Relaxed),
            blob_hits: env
                .telemetry
                .blob_hits
                .load(std::sync::atomic::Ordering::Relaxed),
            embedding_hits: env
                .telemetry
                .embedding_hits
                .load(std::sync::atomic::Ordering::Relaxed),
            total_files: env
                .telemetry
                .total_files
                .load(std::sync::atomic::Ordering::Relaxed),
            total_segments: env
                .telemetry
                .total_segments
                .load(std::sync::atomic::Ordering::Relaxed),
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
    let path = resolve_compatible_cache_path(path);
    let contents = fs::read_to_string(&path)
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
    let path = resolve_compatible_cache_path(path);
    let contents = fs::read_to_string(&path)
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

fn filter_evaluation_helper_documents(
    corpus: LoadedCorpus,
    helper_paths: &[PathBuf],
) -> LoadedCorpus {
    let excluded: HashSet<_> = helper_paths
        .iter()
        .filter(|path| !path.as_os_str().is_empty())
        .cloned()
        .collect();
    if excluded.is_empty() {
        return corpus;
    }

    let original_indexed = corpus.indexed_artifacts;
    let original_skipped = corpus.skipped_artifacts;
    let artifacts: Vec<_> = corpus
        .artifacts
        .into_iter()
        .filter(|document| !excluded.contains(&document.path))
        .collect();
    let removed = original_indexed.saturating_sub(artifacts.len());

    if removed > 0 {
        tracing::info!(
            "→ excluding {} evaluation helper file(s) from corpus",
            removed
        );
    }

    let total_bytes = artifacts
        .iter()
        .map(|document| document.length as u64)
        .sum();

    LoadedCorpus {
        indexed_artifacts: artifacts.len(),
        total_bytes,
        artifacts,
        skipped_artifacts: original_skipped + removed,
    }
}

fn load_agentic_fixture_set(path: &Path) -> Result<AgenticFixtureSet> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read agentic fixtures {}", path.display()))?;
    serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse agentic fixtures {}", path.display()))
}

fn collapse_agentic_task_query(task: &AgenticTaskFixture) -> String {
    task.turns
        .iter()
        .map(|turn| turn.query.trim())
        .filter(|query| !query.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn document_ids_from_emission(emission: &crate::search::SearchEmission) -> Vec<String> {
    match emission {
        crate::search::SearchEmission::Protocol(protocol) => protocol
            .hits
            .iter()
            .map(|hit| search_path_to_document_id(&hit.path))
            .collect(),
        crate::search::SearchEmission::View(view) => view
            .hits
            .iter()
            .map(|hit| search_path_to_document_id(&hit.path))
            .collect(),
        crate::search::SearchEmission::Latent(latent) => latent
            .hits
            .iter()
            .map(|hit| search_path_to_document_id(&hit.path))
            .collect(),
    }
}

fn summarize_latencies(values: &[f64]) -> LatencyMetrics {
    let mut timings = values.to_vec();
    timings.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

    let p50_ms = percentile(&timings, 0.50);
    let p90_ms = percentile(&timings, 0.90);
    let max_ms = timings.last().copied().unwrap_or(0.0);

    LatencyMetrics {
        prepare_ms: 0.0,
        p50_ms,
        p90_ms,
        max_ms,
        target_ms: LATENCY_TARGET_MS,
        p50_over_target_ms: over_target_ms(p50_ms),
        p90_over_target_ms: over_target_ms(p90_ms),
        max_over_target_ms: over_target_ms(max_ms),
    }
}

fn search_path_to_document_id(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(path)
        .to_string()
}

fn expected_documents_satisfied(expected: &[String], actual: &[String]) -> bool {
    if expected.is_empty() {
        return true;
    }

    let actual: HashSet<_> = actual.iter().map(String::as_str).collect();
    expected
        .iter()
        .all(|expected_id| actual.contains(expected_id.as_str()))
}

fn recall_against_expected(hits: &[String], expected: &[String]) -> f64 {
    if expected.is_empty() {
        return 1.0;
    }

    let relevant: HashMap<_, _> = expected
        .iter()
        .map(|document_id| (document_id.clone(), 1_u32))
        .collect();
    recall_at_10(hits, &relevant)
}

fn mrr_at_10(hits: &[String], relevances: &HashMap<String, u32>) -> f64 {
    hits.iter()
        .enumerate()
        .find(|(_, id)| relevances.get(*id).copied().unwrap_or(0) > 0)
        .map(|(index, _)| 1.0 / (index as f64 + 1.0))
        .unwrap_or(0.0)
}

fn recall_at_10(hits: &[String], relevances: &HashMap<String, u32>) -> f64 {
    let relevant_total = relevances.values().filter(|score| **score > 0).count();
    if relevant_total == 0 {
        return 0.0;
    }

    let hits = hits
        .iter()
        .filter(|id| relevances.get(*id).copied().unwrap_or(0) > 0)
        .count();

    hits as f64 / relevant_total as f64
}

fn ndcg_at_10(hits: &[String], relevances: &HashMap<String, u32>) -> f64 {
    let dcg = hits
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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use anyhow::anyhow;

    use super::{filter_evaluation_helper_documents, is_gated_model_error};
    use crate::extract::SourceKind;
    use crate::search::LoadedCorpus;

    #[test]
    fn detects_gated_model_http_status_errors() {
        assert!(is_gated_model_error(&anyhow!("http status: 401")));
        assert!(is_gated_model_error(&anyhow!("http status: 403")));
    }

    #[test]
    fn ignores_non_authentication_errors() {
        assert!(!is_gated_model_error(&anyhow!("failed to parse config")));
    }

    #[test]
    fn filters_eval_helper_documents_from_loaded_corpus() {
        let doc_path = PathBuf::from("/tmp/eval-corpus/doc.txt");
        let queries_path = PathBuf::from("/tmp/eval-corpus/test-queries.tsv");
        let qrels_path = PathBuf::from("/tmp/eval-corpus/qrels/test.tsv");
        let corpus = LoadedCorpus {
            artifacts: vec![
                test_document(&doc_path, "real document"),
                test_document(&queries_path, "helper queries"),
                test_document(&qrels_path, "helper qrels"),
            ],
            total_bytes: ("real document".len() + "helper queries".len() + "helper qrels".len())
                as u64,
            indexed_artifacts: 3,
            skipped_artifacts: 1,
        };

        let filtered =
            filter_evaluation_helper_documents(corpus, &[queries_path.clone(), qrels_path.clone()]);

        assert_eq!(filtered.artifacts.len(), 1);
        assert_eq!(filtered.artifacts[0].path, doc_path);
        assert_eq!(filtered.indexed_artifacts, 1);
        assert_eq!(filtered.skipped_artifacts, 3);
        assert_eq!(filtered.total_bytes, "real document".len() as u64);
    }

    fn test_document(path: &Path, text: &str) -> crate::search::ContextArtifact {
        crate::search::ContextArtifact {
            id: path.display().to_string(),
            kind: crate::search::ContextArtifactKind::File,
            path: path.to_path_buf(),
            source_kind: SourceKind::Text,
            length: text.len(),
            terms: std::collections::HashMap::new(),
            text: text.to_string(),
            segments: Vec::new(),
            provenance: crate::search::ArtifactProvenance {
                adapter: crate::search::AcquisitionAdapterKind::FileSystem,
                source: path.display().to_string(),
                synthetic: false,
            },
            freshness: crate::search::ArtifactFreshness {
                observed_unix_secs: 0,
                modified_unix_secs: None,
            },
            budget: crate::search::ArtifactBudget::from_text(text, 0),
        }
    }
}
