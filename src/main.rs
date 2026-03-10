use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use sift::cache::cache_dir;
use sift::config::Config;
use sift::dense::{DenseModelSpec, DenseReranker};
use sift::eval::{
    LatencyEvaluationRequest, QualityEvaluationRequest, download_scifact_dataset,
    materialize_scifact_dir, render_comparative_report, run_comparative_evaluation,
    run_latency_evaluation, run_quality_evaluation,
};
use sift::search::adapters::qwen::{DEFAULT_QWEN_MODEL_ID, DEFAULT_QWEN_REVISION, QwenModelSpec};
use sift::search::{
    Embedder, FusionPolicy, LocalFileCorpusRepository, OutputFormat, RerankingPolicy,
    RetrieverPolicy, SearchRequest, StrategyPresetRegistry, render_search_response, run_search,
};
use sift::system::Telemetry;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

const SCIFACT_BASE_URL: &str = "https://huggingface.co/datasets/BeIR/scifact/resolve/main";
const SCIFACT_QRELS_BASE_URL: &str =
    "https://huggingface.co/datasets/BeIR/scifact-qrels/resolve/main";

#[derive(Parser)]
#[command(name = "sift")]
#[command(about = "Indexless hybrid search for local retrieval workflows", long_about = None)]
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
    /// Show the applied configuration
    Config,
    /// Search the corpus
    Search(SearchCommand),
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
#[command(override_usage = "sift search [OPTIONS] [PATH] <QUERY>")]
#[command(after_help = "If PATH is omitted, sift searches the current directory.")]
struct SearchCommand {
    #[arg(long)]
    strategy: Option<String>,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    /// Maximum number of results returned.
    limit: Option<usize>,

    /// Number of candidates to score in reranking (defaults to config `shortlist`).
    /// This does not set the final return size; `limit` does.
    #[arg(long)]
    shortlist: Option<usize>,

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
    retrievers: Option<Vec<RetrieverPolicy>>,

    #[arg(long)]
    fusion: Option<FusionPolicy>,

    #[arg(long)]
    reranking: Option<RerankingPolicy>,

    /// Provide QUERY to search the current directory, or PATH QUERY to search a specific corpus.
    #[arg(num_args = 1..=2, value_names = ["PATH", "QUERY"])]
    targets: Vec<String>,
}

impl SearchCommand {
    fn resolve_targets(&self) -> (PathBuf, String) {
        match self.targets.as_slice() {
            [query] => (PathBuf::from("."), query.clone()),
            [path, query] => (PathBuf::from(path), query.clone()),
            _ => unreachable!("clap enforces one or two search targets"),
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
        },
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
    let ignore = sift::config::Ignore::load();
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
        },
        Commands::Config => {
            let toml_string = toml::to_string_pretty(&config)?;
            println!("{}", Config::highlight_toml(&toml_string));
        }
        Commands::Search(search) => {
            let (path, query) = search.resolve_targets();
            let strategy = search
                .strategy
                .unwrap_or_else(|| config.search.strategy.clone());
            let limit = search.limit.unwrap_or(config.search.limit);
            let shortlist = search.shortlist.unwrap_or(config.search.shortlist);

            let spec = DenseModelSpec::with_overrides(
                search
                    .model_id
                    .clone()
                    .or(Some(config.embedding.model_id.clone())),
                search
                    .model_revision
                    .clone()
                    .or(Some(config.embedding.model_revision.clone())),
                search.max_length.or(Some(config.embedding.max_length)),
            );

            let registry = StrategyPresetRegistry::default_registry();
            let plan = registry.resolve(&strategy)?;
            let mut embedder = None;
            if plan.retrievers.contains(&RetrieverPolicy::Vector)
                || search
                    .retrievers
                    .as_ref()
                    .map(|r| r.contains(&RetrieverPolicy::Vector))
                    .unwrap_or(false)
            {
                embedder = Some(Arc::new(DenseReranker::load(spec.clone())?) as Arc<dyn Embedder>);
            }

            let rerank_spec = if search.rerank_model_id.is_some()
                || search.rerank_revision.is_some()
                || plan.reranking == RerankingPolicy::Llm
            {
                Some(QwenModelSpec {
                    model_id: search
                        .rerank_model_id
                        .clone()
                        .or(Some(config.rerank.model_id.clone()))
                        .unwrap_or_else(|| DEFAULT_QWEN_MODEL_ID.to_string()),
                    revision: search
                        .rerank_revision
                        .clone()
                        .or(Some(config.rerank.model_revision.clone()))
                        .unwrap_or_else(|| DEFAULT_QWEN_REVISION.to_string()),
                    max_length: config.rerank.max_length,
                })
            } else {
                None
            };

            let response = run_search(
                &SearchRequest {
                    strategy,
                    query,
                    path,
                    limit,
                    shortlist,
                    dense_model: spec,
                    rerank_model: rerank_spec,
                    verbose: search.verbose,
                    retrievers: search.retrievers.clone(),
                    fusion: search.fusion,
                    reranking: search.reranking,
                    telemetry: telemetry.clone(),
                    cache_dir: None,
                    query_cache: Some(query_cache.clone()),
                },
                Some(&ignore),
                &LocalFileCorpusRepository,
                embedder,
            )?;
            let output = render_search_response(
                &response,
                if search.json {
                    OutputFormat::Json
                } else {
                    OutputFormat::Text
                },
            )?;
            println!("{output}");
        }
    }

    Ok(())
}
