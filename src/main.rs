use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use sift::bench::{
    LatencyBenchmarkRequest, QualityBenchmarkRequest, render_comparative_report,
    run_comparative_benchmark, run_latency_benchmark, run_quality_benchmark,
};
use sift::cache::cache_dir;
use sift::config::Config;
use sift::dense::DenseModelSpec;
use sift::eval::{download_scifact_dataset, materialize_scifact_dir};
use sift::search::{
    FusionPolicy, OutputFormat, RerankingPolicy, RetrieverPolicy, SearchRequest,
    render_search_response, run_search,
};

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
    /// Evaluation corpus utilities
    Eval {
        #[command(subcommand)]
        command: EvalCommands,
    },
    /// Benchmark commands
    Bench {
        #[command(subcommand)]
        command: BenchCommands,
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
    limit: Option<usize>,

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
enum EvalCommands {
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
enum BenchCommands {
    /// Compare all available strategies
    All {
        #[arg(long)]
        corpus: PathBuf,
        #[arg(long)]
        queries: Option<PathBuf>,
        #[arg(long)]
        qrels: PathBuf,
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
    },
    /// Run quality benchmarks
    Quality {
        #[arg(long)]
        strategy: Option<String>,
        #[arg(long)]
        baseline: Option<String>,
        #[arg(long)]
        corpus: PathBuf,
        #[arg(long)]
        queries: Option<PathBuf>,
        #[arg(long)]
        qrels: PathBuf,
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
    },
    /// Run latency benchmarks
    Latency {
        #[arg(long)]
        strategy: Option<String>,
        #[arg(long)]
        corpus: PathBuf,
        #[arg(long)]
        queries: PathBuf,
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
    },
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum Dataset {
    Scifact,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command_line = std::env::args().collect::<Vec<_>>().join(" ");
    let config = Config::load().unwrap_or_default();
    let ignore = sift::config::Ignore::load();

    match cli.command {
        Commands::Eval { command } => match command {
            EvalCommands::Download { dataset, out, verbose: _ } => {
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
            EvalCommands::Materialize {
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
                        .join(format!("{}-materialized", dataset_name))
                });

                match dataset {
                    Dataset::Scifact => {
                        let summary = materialize_scifact_dir(&source, &out)?;
                        println!("{}", serde_json::to_string_pretty(&summary)?);
                    }
                }
            }
        },
        Commands::Bench { command } => match command {
            BenchCommands::All {
                corpus,
                queries,
                qrels,
                shortlist,
                model_id,
                model_revision,
                max_length,
                json,
                verbose,
            } => {
                let shortlist = shortlist.unwrap_or(config.search.shortlist);
                let report = run_comparative_benchmark(
                    &QualityBenchmarkRequest {
                        strategy: String::new(), // Not used for All
                        baseline: None,
                        command: command_line,
                        corpus_dir: corpus,
                        queries_path: queries,
                        qrels_path: qrels,
                        shortlist,
                        dense_model: DenseModelSpec::with_overrides(
                            model_id.clone().or(Some(config.model.model_id.clone())),
                            model_revision.clone().or(Some(config.model.model_revision.clone())),
                            max_length.or(Some(config.model.max_length)),
                        ),
                        verbose,
                    },
                    Some(&ignore),
                )?;

                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    println!("{}", render_comparative_report(&report));
                }
            }
            BenchCommands::Quality {
                strategy,
                baseline,
                corpus,
                queries,
                qrels,
                shortlist,
                model_id,
                model_revision,
                max_length,
                verbose,
            } => {
                let strategy = strategy.unwrap_or_else(|| config.search.strategy.clone());
                let shortlist = shortlist.unwrap_or(config.search.shortlist);
                let report = run_quality_benchmark(
                    &QualityBenchmarkRequest {
                        strategy,
                        baseline,
                        command: command_line,
                        corpus_dir: corpus,
                        queries_path: queries,
                        qrels_path: qrels,
                        shortlist,
                        dense_model: DenseModelSpec::with_overrides(
                            model_id.clone().or(Some(config.model.model_id.clone())),
                            model_revision.clone().or(Some(config.model.model_revision.clone())),
                            max_length.or(Some(config.model.max_length)),
                        ),
                        verbose,
                    },
                    Some(&ignore),
                )?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            BenchCommands::Latency {
                strategy,
                corpus,
                queries,
                shortlist,
                model_id,
                model_revision,
                max_length,
                verbose,
            } => {
                let strategy = strategy.unwrap_or_else(|| config.search.strategy.clone());
                let shortlist = shortlist.unwrap_or(config.search.shortlist);
                let report = run_latency_benchmark(
                    &LatencyBenchmarkRequest {
                        strategy,
                        command: command_line,
                        corpus_dir: corpus,
                        queries_path: queries,
                        shortlist,
                        dense_model: DenseModelSpec::with_overrides(
                            model_id.clone().or(Some(config.model.model_id.clone())),
                            model_revision.clone().or(Some(config.model.model_revision.clone())),
                            max_length.or(Some(config.model.max_length)),
                        ),
                        verbose,
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

            let response = run_search(
                &SearchRequest {
                    strategy,
                    query,
                    path,
                    limit,
                    shortlist,
                    dense_model: DenseModelSpec::with_overrides(
                        search.model_id.clone().or(Some(config.model.model_id.clone())),
                        search
                            .model_revision
                            .clone()
                            .or(Some(config.model.model_revision.clone())),
                        search.max_length.or(Some(config.model.max_length)),
                    ),
                    verbose: search.verbose,
                    retrievers: search.retrievers.clone(),
                    fusion: search.fusion,
                    reranking: search.reranking,
                },
                Some(&ignore),
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
