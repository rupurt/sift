use std::cell::RefCell;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand};
use sift::internal::{
    cache::cache_dir,
    config::{Config, Ignore},
    dense::DenseModelSpec,
    eval::{
        AgenticEvaluationRequest, LatencyEvaluationRequest, QualityEvaluationRequest,
        download_scifact_dataset, materialize_scifact_dir, render_comparative_report,
        run_agentic_evaluation, run_comparative_evaluation, run_latency_evaluation,
        run_quality_evaluation,
    },
    optimize::{OptimizeRequest, run_optimization},
    search::{
        OutputFormat,
        adapters::gemma::GemmaModelSpec,
        adapters::qwen::{DEFAULT_QWEN_MODEL_ID, DEFAULT_QWEN_REVISION, QwenModelSpec},
        render_autonomous_search_response, render_search_response,
    },
    system::Telemetry,
};
use sift::{
    AutonomousPlannerStrategy, AutonomousPlannerStrategyKind, AutonomousSearchMode,
    AutonomousSearchRequest, Fusion, Reranking, Retriever, SearchInput, SearchOptions,
    SearchProgress, SearchTelemetry, Sift,
};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[cfg(test)]
mod versioning;

#[cfg(test)]
mod search_cli_tests {
    use super::*;

    #[test]
    fn progress_renderer_includes_indexing_cache_metrics() {
        let mut renderer = ProgressRenderer::new(Vec::new(), true);
        let telemetry = SearchTelemetry {
            heuristic_hits: 3,
            blob_hits: 7,
            fresh_artifact_builds: 2,
            skipped_artifacts: 1,
            embedding_hits: 0,
            total_files: 10,
            total_segments: 24,
            bm25_index_cache_hits: 0,
            bm25_index_builds: 1,
            sector_cache_hits: 4,
            sector_rebuilds: 1,
            sector_shard_cache_hits: 3,
            sector_shard_builds: 1,
        };

        renderer
            .update(
                &SearchProgress::Indexing {
                    phase: sift::SearchPhase::Indexing,
                    files_processed: 4,
                    files_total: 10,
                    estimated_remaining: Some(Duration::from_secs(12)),
                },
                &telemetry,
            )
            .expect("render indexing progress");
        renderer.finish().expect("finish renderer");

        let output =
            String::from_utf8(renderer.into_inner()).expect("progress output should be utf-8");
        assert!(output.contains("Indexing 4/10 files"));
        assert!(output.contains("blobs 7"));
        assert!(output.contains("fresh 2"));
        assert!(output.contains("sector cache 4 rebuild 1"));
        assert!(output.contains("sector bm25 cache 3 build 1"));
        assert!(output.contains("bm25 cache 0 build 1"));
    }

    #[test]
    fn direct_search_targets_remain_query_driven_without_agent() {
        let cli = Cli::try_parse_from(["sift", "search", "./docs", "cache invalidation"])
            .expect("parse direct search command");

        let Commands::Search(search) = cli.command else {
            panic!("expected search command");
        };

        let (path, query) = search
            .resolve_direct_targets()
            .expect("resolve direct search targets");
        assert_eq!(path, PathBuf::from("./docs"));
        assert_eq!(query, "cache invalidation");
    }

    #[test]
    fn agent_search_builds_heuristic_autonomous_request() {
        let cli = Cli::try_parse_from(["sift", "search", "--agent", "Search Me", "./docs"])
            .expect("parse agent search command");

        let Commands::Search(search) = cli.command else {
            panic!("expected search command");
        };

        let request = search
            .to_autonomous_request()
            .expect("build autonomous request");
        assert_eq!(request.path, PathBuf::from("./docs"));
        assert_eq!(request.root_task, "Search Me");
        assert_eq!(
            request.planner_strategy,
            AutonomousPlannerStrategy::heuristic()
        );
        assert_eq!(request.mode, AutonomousSearchMode::Linear);
    }

    #[test]
    fn agent_search_can_select_model_driven_strategy_and_profile() {
        let cli = Cli::try_parse_from([
            "sift",
            "search",
            "--agent",
            "Search Me",
            "--planner-strategy",
            "model-driven",
            "--planner-profile",
            "local-planner-v1",
        ])
        .expect("parse model-driven agent search command");

        let Commands::Search(search) = cli.command else {
            panic!("expected search command");
        };

        let request = search
            .to_autonomous_request()
            .expect("build model-driven autonomous request");
        assert_eq!(
            request.planner_strategy,
            AutonomousPlannerStrategy::model_driven().with_profile("local-planner-v1")
        );
    }

    #[test]
    fn agent_search_can_select_graph_mode() {
        let cli = Cli::try_parse_from([
            "sift",
            "search",
            "--agent",
            "Search Me",
            "--agent-mode",
            "graph",
            "./docs",
        ])
        .expect("parse graph agent search command");

        let Commands::Search(search) = cli.command else {
            panic!("expected search command");
        };

        let request = search
            .to_autonomous_request()
            .expect("build graph autonomous request");
        assert_eq!(request.path, PathBuf::from("./docs"));
        assert_eq!(request.root_task, "Search Me");
        assert_eq!(request.mode, AutonomousSearchMode::Graph);
        assert_eq!(
            request.planner_strategy,
            AutonomousPlannerStrategy::heuristic()
        );
    }
}

const SCIFACT_BASE_URL: &str = "https://huggingface.co/datasets/BeIR/scifact/resolve/main";
const SCIFACT_QRELS_BASE_URL: &str =
    "https://huggingface.co/datasets/BeIR/scifact-qrels/resolve/main";
const CLI_VERSION: &str = env!("SIFT_CLI_VERSION");

#[derive(Parser)]
#[command(name = "sift")]
#[command(about = "Hybrid and agentic local search for retrieval workflows", long_about = None)]
#[command(version = CLI_VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluation dataset management
    Dataset {
        #[command(subcommand)]
        command: DatasetCommands,
    },
    /// Run evaluations and quality measurements
    Eval {
        #[command(subcommand)]
        command: EvalCommands,
    },
    /// Auto-tune prompts to maximize Signal Gain
    Optimize {
        #[arg(long)]
        dataset: Option<Dataset>,
        #[arg(long)]
        corpus: Option<PathBuf>,
        #[arg(long)]
        queries: Option<PathBuf>,
        #[arg(long)]
        qrels: Option<PathBuf>,
        #[arg(long, default_value_t = 3)]
        iterations: usize,
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
        #[arg(long)]
        query_limit: Option<usize>,
    },
    /// Show the applied configuration
    Config,
    /// Search the corpus
    Search(SearchCommand),
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
#[command(
    override_usage = "sift search [OPTIONS] [PATH] <QUERY>\n       sift search [OPTIONS] [PATH] --agent <ROOT_TASK>"
)]
#[command(
    after_help = "If PATH is omitted, sift searches the current directory.\nUse --agent to run the shared autonomous planner runtime with ROOT_TASK."
)]
struct SearchCommand {
    #[arg(long)]
    strategy: Option<String>,

    #[arg(long, value_name = "ROOT_TASK")]
    /// Run the shared autonomous planner runtime with ROOT_TASK.
    agent: Option<String>,

    #[arg(long, value_enum, requires = "agent")]
    /// Autonomous runtime mode for agent search.
    agent_mode: Option<SearchAgentMode>,

    #[arg(long, value_enum, requires = "agent")]
    /// Built-in planner strategy for agent mode.
    planner_strategy: Option<AutonomousPlannerStrategyKind>,

    #[arg(long, requires = "agent")]
    /// Optional planner profile, typically used with model-driven planning.
    planner_profile: Option<String>,

    #[arg(long)]
    /// Explicit intent context to help guide search and ranking.
    intent: Option<String>,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    /// Maximum number of results returned.
    limit: Option<usize>,

    /// Number of candidates to score in reranking (defaults to config `shortlist`).
    /// This does not set the final return size; `limit` does.
    #[arg(long)]
    shortlist: Option<usize>,

    /// Timeout in milliseconds for each retriever strategy before proceeding with partial results.
    #[arg(long)]
    retriever_timeout_ms: Option<u64>,

    #[arg(long)]
    model_id: Option<String>,

    #[arg(long)]
    model_revision: Option<String>,

    #[arg(long)]
    rerank_model_id: Option<String>,

    #[arg(long)]
    rerank_revision: Option<String>,

    #[arg(long)]
    max_length: Option<usize>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(long, value_delimiter = ',', num_args = 1..)]
    retrievers: Option<Vec<SearchRetriever>>,

    #[arg(long)]
    fusion: Option<SearchFusion>,

    #[arg(long)]
    reranking: Option<SearchReranking>,

    /// Provide QUERY to search the current directory, PATH QUERY to search a specific corpus,
    /// or PATH when using --agent.
    #[arg(num_args = 0..=2, value_names = ["PATH", "QUERY"])]
    targets: Vec<String>,
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum SearchRetriever {
    #[value(name = "bm25")]
    Bm25,
    #[value(name = "phrase")]
    Phrase,
    #[value(name = "vector")]
    Vector,
}

impl From<SearchRetriever> for Retriever {
    fn from(value: SearchRetriever) -> Self {
        match value {
            SearchRetriever::Bm25 => Retriever::Bm25,
            SearchRetriever::Phrase => Retriever::Phrase,
            SearchRetriever::Vector => Retriever::Vector,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum SearchFusion {
    #[value(name = "rrf")]
    Rrf,
}

impl From<SearchFusion> for Fusion {
    fn from(value: SearchFusion) -> Self {
        match value {
            SearchFusion::Rrf => Fusion::Rrf,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum SearchReranking {
    #[value(name = "none")]
    None,
    #[value(name = "position-aware")]
    PositionAware,
    #[value(name = "llm")]
    Llm,
    #[value(name = "jina")]
    Jina,
    #[value(name = "gemma")]
    Gemma,
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum SearchAgentMode {
    #[value(name = "linear")]
    Linear,
    #[value(name = "graph")]
    Graph,
}

impl From<SearchAgentMode> for AutonomousSearchMode {
    fn from(value: SearchAgentMode) -> Self {
        match value {
            SearchAgentMode::Linear => AutonomousSearchMode::Linear,
            SearchAgentMode::Graph => AutonomousSearchMode::Graph,
        }
    }
}

impl From<SearchReranking> for Reranking {
    fn from(value: SearchReranking) -> Self {
        match value {
            SearchReranking::None => Reranking::None,
            SearchReranking::PositionAware => Reranking::PositionAware,
            SearchReranking::Llm => Reranking::Llm,
            SearchReranking::Jina => Reranking::Jina,
            SearchReranking::Gemma => Reranking::Gemma,
        }
    }
}

impl SearchCommand {
    fn resolve_direct_targets(&self) -> Result<(PathBuf, String)> {
        match self.targets.as_slice() {
            [query] => Ok((PathBuf::from("."), query.clone())),
            [path, query] => Ok((PathBuf::from(path), query.clone())),
            _ => bail!("direct search expects QUERY or PATH QUERY"),
        }
    }

    fn resolve_agent_target(&self) -> Result<(PathBuf, String)> {
        let root_task = self
            .agent
            .clone()
            .ok_or_else(|| anyhow::anyhow!("agent search requires --agent <ROOT_TASK>"))?;

        match self.targets.as_slice() {
            [] => Ok((PathBuf::from("."), root_task)),
            [path] => Ok((PathBuf::from(path), root_task)),
            _ => bail!("agent search expects PATH --agent <ROOT_TASK> or --agent <ROOT_TASK>"),
        }
    }

    fn output_format(&self) -> OutputFormat {
        if self.json {
            OutputFormat::Json
        } else {
            OutputFormat::Text
        }
    }

    fn to_input(&self, config: &Config) -> Result<SearchInput> {
        let (path, query) = self.resolve_direct_targets()?;
        let mut options = SearchOptions::default().with_verbose(self.verbose);

        if let Some(strategy) = &self.strategy {
            options = options.with_strategy(strategy.clone());
        }
        if let Some(intent) = &self.intent {
            options = options.with_intent(intent.clone());
        }
        if let Some(limit) = self.limit {
            options = options.with_limit(limit);
        }
        if let Some(shortlist) = self.shortlist {
            options = options.with_shortlist(shortlist);
        }
        if let Some(retriever_timeout_ms) = self.retriever_timeout_ms {
            options = options.with_retriever_timeout_ms(retriever_timeout_ms);
        }
        if let Some(dense_model) = self.resolve_dense_model(config) {
            options = options.with_dense_model(dense_model);
        }
        if let Some(rerank_model) = self.resolve_rerank_model(config) {
            options = options.with_rerank_model(rerank_model);
        }
        options = options.with_gemma_model(self.resolve_gemma_model(config));
        if let Some(retrievers) = &self.retrievers {
            options = options.with_retrievers(retrievers.iter().copied().map(Into::into).collect());
        }
        if let Some(fusion) = self.fusion {
            options = options.with_fusion(fusion.into());
        }
        if let Some(reranking) = self.reranking {
            options = options.with_reranking(reranking.into());
        }

        Ok(SearchInput::new(path, query).with_options(options))
    }

    fn to_autonomous_request(&self) -> Result<AutonomousSearchRequest> {
        self.ensure_supported_agent_options()?;
        let (path, root_task) = self.resolve_agent_target()?;
        let mut request = AutonomousSearchRequest::new(path, root_task)
            .with_planner_strategy(self.resolve_agent_planner_strategy())
            .with_mode(
                self.agent_mode
                    .map(Into::into)
                    .unwrap_or(AutonomousSearchMode::Linear),
            )
            .with_verbose(self.verbose);

        if let Some(strategy) = &self.strategy {
            request = request.with_strategy(strategy.clone());
        }
        if let Some(intent) = &self.intent {
            request = request.with_intent(intent.clone());
        }
        if let Some(limit) = self.limit {
            request = request.with_limit(limit);
        }
        if let Some(shortlist) = self.shortlist {
            request = request.with_shortlist(shortlist);
        }

        Ok(request)
    }

    fn resolve_agent_planner_strategy(&self) -> AutonomousPlannerStrategy {
        let kind = match (self.planner_strategy, self.planner_profile.as_ref()) {
            (Some(kind), _) => kind,
            (None, Some(_)) => AutonomousPlannerStrategyKind::ModelDriven,
            (None, None) => AutonomousPlannerStrategyKind::Heuristic,
        };

        let mut strategy = match kind {
            AutonomousPlannerStrategyKind::Heuristic => AutonomousPlannerStrategy::heuristic(),
            AutonomousPlannerStrategyKind::ModelDriven => AutonomousPlannerStrategy::model_driven(),
        };

        if let Some(profile) = &self.planner_profile {
            strategy = strategy.with_profile(profile.clone());
        }

        strategy
    }

    fn ensure_supported_agent_options(&self) -> Result<()> {
        let mut unsupported = Vec::new();
        if self.model_id.is_some() || self.model_revision.is_some() || self.max_length.is_some() {
            unsupported.push("--model-id/--model-revision/--max-length");
        }
        if self.rerank_model_id.is_some() || self.rerank_revision.is_some() {
            unsupported.push("--rerank-model-id/--rerank-revision");
        }
        if self.retrievers.is_some() {
            unsupported.push("--retrievers");
        }
        if self.retriever_timeout_ms.is_some() {
            unsupported.push("--retriever-timeout-ms");
        }
        if self.fusion.is_some() {
            unsupported.push("--fusion");
        }
        if self.reranking.is_some() {
            unsupported.push("--reranking");
        }
        if unsupported.is_empty() {
            return Ok(());
        }

        bail!(
            "agent search currently supports --strategy, --agent-mode, --planner-strategy, --planner-profile, --intent, --limit, --shortlist, --json, and verbosity flags; unsupported with --agent: {}",
            unsupported.join(", ")
        )
    }

    fn resolve_dense_model(&self, config: &Config) -> Option<DenseModelSpec> {
        if self.model_id.is_none() && self.model_revision.is_none() && self.max_length.is_none() {
            return None;
        }

        Some(DenseModelSpec::with_overrides(
            self.model_id
                .clone()
                .or(Some(config.embedding.model_id.clone())),
            self.model_revision
                .clone()
                .or(Some(config.embedding.model_revision.clone())),
            self.max_length.or(Some(config.embedding.max_length)),
        ))
    }

    fn resolve_rerank_model(&self, config: &Config) -> Option<QwenModelSpec> {
        if self.rerank_model_id.is_none() && self.rerank_revision.is_none() {
            return None;
        }

        Some(QwenModelSpec {
            model_id: self
                .rerank_model_id
                .clone()
                .or(Some(config.rerank.model_id.clone()))
                .unwrap_or_else(|| DEFAULT_QWEN_MODEL_ID.to_string()),
            revision: self
                .rerank_revision
                .clone()
                .or(Some(config.rerank.model_revision.clone()))
                .unwrap_or_else(|| DEFAULT_QWEN_REVISION.to_string()),
            max_length: config.rerank.max_length,
        })
    }

    fn resolve_gemma_model(&self, config: &Config) -> GemmaModelSpec {
        GemmaModelSpec {
            model_id: config.gemma.model_id.clone(),
            revision: config.gemma.model_revision.clone(),
            max_length: config.gemma.max_length,
        }
    }
}

#[derive(Subcommand)]
enum DatasetCommands {
    /// Download an evaluation dataset
    Download {
        dataset: Dataset,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
    },
    /// Materialize a downloaded dataset as local text files
    Materialize {
        dataset: Dataset,
        #[arg(long)]
        source: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
    },
}

#[derive(Subcommand)]
enum EvalCommands {
    /// Compare all available strategies
    All {
        #[arg(long)]
        dataset: Option<Dataset>,
        #[arg(long)]
        corpus: Option<PathBuf>,
        #[arg(long)]
        queries: Option<PathBuf>,
        #[arg(long)]
        qrels: Option<PathBuf>,
        #[arg(long)]
        shortlist: Option<usize>,
        #[arg(long)]
        model_id: Option<String>,
        #[arg(long)]
        model_revision: Option<String>,
        #[arg(long)]
        max_length: Option<usize>,
        #[arg(long)]
        json: bool,
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
        #[arg(long)]
        query_limit: Option<usize>,
    },
    /// Run quality evaluations
    Quality {
        #[arg(long)]
        strategy: Option<String>,
        #[arg(long)]
        baseline: Option<String>,
        #[arg(long)]
        dataset: Option<Dataset>,
        #[arg(long)]
        corpus: Option<PathBuf>,
        #[arg(long)]
        queries: Option<PathBuf>,
        #[arg(long)]
        qrels: Option<PathBuf>,
        #[arg(long)]
        shortlist: Option<usize>,
        #[arg(long)]
        model_id: Option<String>,
        #[arg(long)]
        model_revision: Option<String>,
        #[arg(long)]
        max_length: Option<usize>,
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
        #[arg(long)]
        query_limit: Option<usize>,
    },
    /// Run latency measurements
    Latency {
        #[arg(long)]
        strategy: Option<String>,
        #[arg(long)]
        dataset: Option<Dataset>,
        #[arg(long)]
        corpus: Option<PathBuf>,
        #[arg(long)]
        queries: Option<PathBuf>,
        #[arg(long)]
        shortlist: Option<usize>,
        #[arg(long)]
        model_id: Option<String>,
        #[arg(long)]
        model_revision: Option<String>,
        #[arg(long)]
        max_length: Option<usize>,
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
        #[arg(long)]
        query_limit: Option<usize>,
    },
    /// Run planned multi-turn agentic evaluation fixtures
    Agentic {
        #[arg(long)]
        strategy: Option<String>,
        #[arg(long)]
        corpus: PathBuf,
        #[arg(long)]
        fixtures: PathBuf,
        #[arg(long)]
        shortlist: Option<usize>,
        #[arg(long)]
        retained_artifact_limit: Option<usize>,
        #[arg(long)]
        model_id: Option<String>,
        #[arg(long)]
        model_revision: Option<String>,
        #[arg(long)]
        max_length: Option<usize>,
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
    },
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum Dataset {
    Scifact,
}

fn resolve_eval_paths(
    dataset: Option<Dataset>,
    corpus: Option<PathBuf>,
    qrels: Option<PathBuf>,
    queries: Option<PathBuf>,
) -> Result<(PathBuf, Option<PathBuf>, Option<PathBuf>)> {
    match (dataset, corpus, qrels) {
        (Some(Dataset::Scifact), None, None) => {
            let base = cache_dir("eval")?.join("scifact");
            let corpus = cache_dir("eval")?.join("scifact-files");
            let qrels = base.join("qrels").join("test.tsv");
            let queries = queries.or_else(|| Some(corpus.join("test-queries.tsv")));
            Ok((corpus, Some(qrels), queries))
        }
        (None, Some(c), q) => Ok((c, q, queries)),
        (Some(_), Some(c), q) => Ok((c, q, queries)),
        _ => anyhow::bail!("either --dataset or --corpus must be provided"),
    }
}

struct ProgressRenderer<W: Write> {
    writer: W,
    interactive: bool,
    last_line: Option<String>,
}

impl<W: Write> ProgressRenderer<W> {
    fn new(writer: W, interactive: bool) -> Self {
        Self {
            writer,
            interactive,
            last_line: None,
        }
    }

    fn update(&mut self, progress: &SearchProgress, telemetry: &SearchTelemetry) -> Result<()> {
        if !self.interactive {
            return Ok(());
        }

        let Some(line) = Self::format_line(progress, telemetry) else {
            return Ok(());
        };

        if self.last_line.as_ref() == Some(&line) {
            return Ok(());
        }

        let width = self
            .last_line
            .as_ref()
            .map(|previous| previous.len())
            .unwrap_or_default()
            .max(line.len());
        write!(self.writer, "\r{line:<width$}")?;
        self.writer.flush()?;
        self.last_line = Some(line);
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        if self.interactive && self.last_line.is_some() {
            writeln!(self.writer)?;
            self.writer.flush()?;
            self.last_line = None;
        }
        Ok(())
    }

    #[cfg(test)]
    fn into_inner(self) -> W {
        self.writer
    }

    fn format_line(progress: &SearchProgress, telemetry: &SearchTelemetry) -> Option<String> {
        match progress {
            SearchProgress::Indexing {
                files_processed,
                files_total,
                estimated_remaining,
                ..
            } => Some(format!(
                "Indexing {files_processed}/{files_total} files | blobs {} | fresh {} | skipped {} | segments {} | sector cache {} rebuild {} | sector bm25 cache {} build {} | bm25 cache {} build {}{}",
                telemetry.blob_hits,
                telemetry.fresh_artifact_builds,
                telemetry.skipped_artifacts,
                telemetry.total_segments,
                telemetry.sector_cache_hits,
                telemetry.sector_rebuilds,
                telemetry.sector_shard_cache_hits,
                telemetry.sector_shard_builds,
                telemetry.bm25_index_cache_hits,
                telemetry.bm25_index_builds,
                Self::format_eta(*estimated_remaining),
            )),
            SearchProgress::Embedding {
                chunks_processed,
                chunks_total,
                estimated_remaining,
                ..
            } => Some(format!(
                "Embedding {chunks_processed}/{chunks_total} chunks{}",
                Self::format_eta(*estimated_remaining),
            )),
            SearchProgress::PlannerStep {
                step_index,
                action,
                query,
                estimated_remaining,
                ..
            } => {
                let query_suffix = query
                    .as_ref()
                    .map(|query| format!(" | query {query}"))
                    .unwrap_or_default();
                Some(format!(
                    "Planning step {} | {}{}{}",
                    step_index + 1,
                    action,
                    query_suffix,
                    Self::format_eta(*estimated_remaining),
                ))
            }
            SearchProgress::Retrieving {
                turn_index,
                turns_total,
                estimated_remaining,
                ..
            } => Some(format!(
                "Retrieving turn {}/{}{}",
                turn_index + 1,
                turns_total,
                Self::format_eta(*estimated_remaining),
            )),
            SearchProgress::Ranking {
                results_processed,
                results_total,
                estimated_remaining,
                ..
            } => Some(format!(
                "Ranking {results_processed}/{results_total} results{}",
                Self::format_eta(*estimated_remaining),
            )),
        }
    }

    fn format_eta(estimated_remaining: Option<Duration>) -> String {
        estimated_remaining
            .map(|remaining| format!(" | eta {}", Self::format_duration(remaining)))
            .unwrap_or_default()
    }

    fn format_duration(duration: Duration) -> String {
        let seconds = duration.as_secs();
        match seconds {
            0 => "<1s".to_string(),
            1..=59 => format!("{seconds}s"),
            60..=3599 => format!("{}m{}s", seconds / 60, seconds % 60),
            _ => format!("{}h{}m", seconds / 3600, (seconds % 3600) / 60),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let verbose = match &cli.command {
        Commands::Dataset { command } => match command {
            DatasetCommands::Download { verbose, .. } => *verbose,
            DatasetCommands::Materialize { verbose, .. } => *verbose,
        },
        Commands::Eval { command } => match command {
            EvalCommands::All { verbose, .. } => *verbose,
            EvalCommands::Quality { verbose, .. } => *verbose,
            EvalCommands::Latency { verbose, .. } => *verbose,
            EvalCommands::Agentic { verbose, .. } => *verbose,
        },
        Commands::Optimize { verbose, .. } => *verbose,
        Commands::Search(search) => search.verbose,
        Commands::Config => 0,
    };

    let filter = match verbose {
        0 => EnvFilter::new("off"),
        1 => EnvFilter::new("info"),
        2 => EnvFilter::new("debug"),
        _ => EnvFilter::new("trace"),
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr).with_target(false))
        .with(filter)
        .init();

    let command_line = std::env::args().collect::<Vec<_>>().join(" ");
    let config = Config::load().unwrap_or_default();
    let ignore = Ignore::load();
    let telemetry = Arc::new(Telemetry::new());
    let query_cache = Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));

    match cli.command {
        Commands::Dataset { command } => match command {
            DatasetCommands::Download {
                dataset,
                out,
                verbose: _,
            } => {
                let dataset_name = match dataset {
                    Dataset::Scifact => "scifact",
                };
                let out = out.unwrap_or_else(|| {
                    cache_dir("eval")
                        .expect("resolve eval cache dir")
                        .join(dataset_name)
                });

                match dataset {
                    Dataset::Scifact => {
                        let summary = download_scifact_dataset(
                            SCIFACT_BASE_URL,
                            SCIFACT_QRELS_BASE_URL,
                            &out,
                        )?;
                        println!("{}", serde_json::to_string_pretty(&summary)?);
                    }
                }
            }
            DatasetCommands::Materialize {
                dataset,
                source,
                out,
                verbose: _,
            } => {
                let dataset_name = match dataset {
                    Dataset::Scifact => "scifact",
                };
                let source = source.unwrap_or_else(|| {
                    cache_dir("eval")
                        .expect("resolve eval cache dir")
                        .join(dataset_name)
                });
                let out = out.unwrap_or_else(|| {
                    cache_dir("eval")
                        .expect("resolve eval cache dir")
                        .join(format!("{}-files", dataset_name))
                });

                match dataset {
                    Dataset::Scifact => {
                        let summary = materialize_scifact_dir(&source, &out)?;
                        println!("{}", serde_json::to_string_pretty(&summary)?);
                    }
                }
            }
        },
        Commands::Eval { command } => match command {
            EvalCommands::All {
                dataset,
                corpus,
                queries,
                qrels,
                shortlist,
                model_id,
                model_revision,
                max_length,
                json,
                verbose,
                query_limit,
            } => {
                let (corpus, qrels, queries) = resolve_eval_paths(dataset, corpus, qrels, queries)?;
                let shortlist = shortlist.unwrap_or(config.search.shortlist);
                let report = run_comparative_evaluation(
                    &QualityEvaluationRequest {
                        strategy: String::new(), // Not used for All
                        baseline: None,
                        command: command_line,
                        corpus_dir: corpus,
                        queries_path: queries,
                        qrels_path: qrels.ok_or_else(|| {
                            anyhow::anyhow!("qrels file must be provided for quality evaluation")
                        })?,
                        shortlist,
                        dense_model: DenseModelSpec::with_overrides(
                            model_id.clone().or(Some(config.embedding.model_id.clone())),
                            model_revision
                                .clone()
                                .or(Some(config.embedding.model_revision.clone())),
                            max_length.or(Some(config.embedding.max_length)),
                        ),
                        verbose,
                        query_limit,
                        prompts: Some(config.prompts.clone()),
                    },
                    Some(&ignore),
                )?;

                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    println!("{}", render_comparative_report(&report));
                }
            }
            EvalCommands::Quality {
                strategy,
                baseline,
                dataset,
                corpus,
                queries,
                qrels,
                shortlist,
                model_id,
                model_revision,
                max_length,
                verbose,
                query_limit,
            } => {
                let (corpus, qrels, queries) = resolve_eval_paths(dataset, corpus, qrels, queries)?;
                let strategy = strategy.unwrap_or_else(|| config.search.strategy.clone());
                let shortlist = shortlist.unwrap_or(config.search.shortlist);
                let report = run_quality_evaluation(
                    &QualityEvaluationRequest {
                        strategy,
                        baseline,
                        command: command_line,
                        corpus_dir: corpus,
                        queries_path: queries,
                        qrels_path: qrels.ok_or_else(|| {
                            anyhow::anyhow!("qrels file must be provided for quality evaluation")
                        })?,
                        shortlist,
                        dense_model: DenseModelSpec::with_overrides(
                            model_id.clone().or(Some(config.embedding.model_id.clone())),
                            model_revision
                                .clone()
                                .or(Some(config.embedding.model_revision.clone())),
                            max_length.or(Some(config.embedding.max_length)),
                        ),
                        verbose,
                        query_limit,
                        prompts: Some(config.prompts.clone()),
                    },
                    Some(&ignore),
                )?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            EvalCommands::Latency {
                strategy,
                dataset,
                corpus,
                queries,
                shortlist,
                model_id,
                model_revision,
                max_length,
                verbose,
                query_limit,
            } => {
                let (corpus, _, queries) = resolve_eval_paths(dataset, corpus, None, queries)?;
                let strategy = strategy.unwrap_or_else(|| config.search.strategy.clone());
                let shortlist = shortlist.unwrap_or(config.search.shortlist);
                let report = run_latency_evaluation(
                    &LatencyEvaluationRequest {
                        strategy,
                        command: command_line,
                        corpus_dir: corpus,
                        queries_path: queries.ok_or_else(|| {
                            anyhow::anyhow!("queries file must be provided for latency evaluation")
                        })?,
                        shortlist,
                        dense_model: DenseModelSpec::with_overrides(
                            model_id.clone().or(Some(config.embedding.model_id.clone())),
                            model_revision
                                .clone()
                                .or(Some(config.embedding.model_revision.clone())),
                            max_length.or(Some(config.embedding.max_length)),
                        ),
                        verbose,
                        query_limit,
                    },
                    Some(&ignore),
                )?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            EvalCommands::Agentic {
                strategy,
                corpus,
                fixtures,
                shortlist,
                retained_artifact_limit,
                model_id,
                model_revision,
                max_length,
                verbose,
            } => {
                let strategy = strategy.unwrap_or_else(|| config.search.strategy.clone());
                let shortlist = shortlist.unwrap_or(config.search.shortlist);
                let report = run_agentic_evaluation(
                    &AgenticEvaluationRequest {
                        strategy,
                        baseline_strategy: None,
                        planner_strategy: sift::AutonomousPlannerStrategy::default(),
                        command: command_line,
                        corpus_dir: corpus,
                        fixtures_path: fixtures,
                        shortlist,
                        dense_model: DenseModelSpec::with_overrides(
                            model_id.clone().or(Some(config.embedding.model_id.clone())),
                            model_revision
                                .clone()
                                .or(Some(config.embedding.model_revision.clone())),
                            max_length.or(Some(config.embedding.max_length)),
                        ),
                        retained_artifact_limit: retained_artifact_limit.unwrap_or(1),
                        verbose,
                        prompts: Some(config.prompts.clone()),
                    },
                    Some(&ignore),
                )?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
        },
        Commands::Optimize {
            dataset,
            corpus,
            queries,
            qrels,
            iterations,
            verbose,
            query_limit,
        } => {
            let (corpus, qrels, queries) = resolve_eval_paths(dataset, corpus, qrels, queries)?;
            let req = OptimizeRequest {
                corpus_dir: corpus,
                queries_path: queries.ok_or_else(|| {
                    anyhow::anyhow!("queries file must be provided for optimization")
                })?,
                qrels_path: qrels.ok_or_else(|| {
                    anyhow::anyhow!("qrels file must be provided for optimization")
                })?,
                shortlist: config.search.shortlist,
                iterations,
                command: command_line,
                verbose,
                query_limit,
            };
            run_optimization(&req, Some(&ignore), &config)?;
        }
        Commands::Config => {
            let toml_string = toml::to_string_pretty(&config)?;
            println!("{}", Config::highlight_toml(&toml_string));
        }
        Commands::Search(search) => {
            let direct_input = if search.agent.is_some() {
                None
            } else {
                Some(search.to_input(&config)?)
            };
            let progress_enabled =
                search.output_format() == OutputFormat::Text && std::io::stderr().is_terminal();
            let progress = RefCell::new(ProgressRenderer::new(std::io::stderr(), progress_enabled));
            let engine = Sift::builder()
                .with_config(config)
                .with_ignore(ignore)
                .with_telemetry(telemetry)
                .with_query_cache(query_cache)
                .with_cache_dir(cache_dir("search")?)
                .build();
            let output = if search.agent.is_some() {
                let response = engine.search_autonomous_with_progress(
                    search.to_autonomous_request()?,
                    Some(|event: &SearchProgress| {
                        let telemetry = engine.telemetry_snapshot();
                        let _ = progress.borrow_mut().update(event, &telemetry);
                    }),
                );
                let _ = progress.borrow_mut().finish();
                let response = response?;
                render_autonomous_search_response(&response, search.output_format())?
            } else {
                let response = engine.search_with_progress(
                    direct_input.expect("direct search input should be built"),
                    Some(|event: &SearchProgress| {
                        let telemetry = engine.telemetry_snapshot();
                        let _ = progress.borrow_mut().update(event, &telemetry);
                    }),
                );
                let _ = progress.borrow_mut().finish();
                let response = response?;
                render_search_response(&response, search.output_format())?
            };
            println!("{output}");
        }
    }

    Ok(())
}
