use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use sift::bench::{
    LatencyBenchmarkRequest, QualityBenchmarkRequest, run_latency_benchmark, run_quality_benchmark,
};
use sift::dense::DenseModelSpec;
use sift::eval::{download_scifact_dataset, materialize_scifact_dir};
use sift::search::{
    DEFAULT_HYBRID_SHORTLIST, DEFAULT_RESULT_LIMIT, OutputFormat, SearchRequest,
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
    /// Search the corpus
    Search(SearchCommand),
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
#[command(override_usage = "sift search [OPTIONS] [PATH] <QUERY>")]
#[command(after_help = "If PATH is omitted, sift searches the current directory.")]
struct SearchCommand {
    #[arg(long, default_value = "hybrid")]
    strategy: String,

    #[arg(long)]
    json: bool,

    #[arg(long, default_value_t = DEFAULT_RESULT_LIMIT)]
    limit: usize,

    #[arg(long, default_value_t = DEFAULT_HYBRID_SHORTLIST)]
    shortlist: usize,

    #[arg(long)]
    model_id: Option<String>,

    #[arg(long)]
    model_revision: Option<String>,

    #[arg(long)]
    max_length: Option<usize>,

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
        out: PathBuf,
    },
    /// Materialize a downloaded dataset as local text files
    Materialize {
        dataset: Dataset,
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Subcommand)]
enum BenchCommands {
    /// Run quality benchmarks
    Quality {
        #[arg(long, default_value = "bm25")]
        strategy: String,
        #[arg(long)]
        baseline: Option<String>,
        #[arg(long)]
        corpus: PathBuf,
        #[arg(long)]
        queries: Option<PathBuf>,
        #[arg(long)]
        qrels: PathBuf,
        #[arg(long, default_value_t = DEFAULT_HYBRID_SHORTLIST)]
        shortlist: usize,
        #[arg(long)]
        model_id: Option<String>,
        #[arg(long)]
        model_revision: Option<String>,
        #[arg(long)]
        max_length: Option<usize>,
    },
    /// Run latency benchmarks
    Latency {
        #[arg(long, default_value = "bm25")]
        strategy: String,
        #[arg(long)]
        corpus: PathBuf,
        #[arg(long)]
        queries: PathBuf,
        #[arg(long, default_value_t = DEFAULT_HYBRID_SHORTLIST)]
        shortlist: usize,
        #[arg(long)]
        model_id: Option<String>,
        #[arg(long)]
        model_revision: Option<String>,
        #[arg(long)]
        max_length: Option<usize>,
    },
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum Dataset {
    Scifact,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command_line = std::env::args().collect::<Vec<_>>().join(" ");

    match cli.command {
        Commands::Eval { command } => match command {
            EvalCommands::Download { dataset, out } => match dataset {
                Dataset::Scifact => {
                    let summary =
                        download_scifact_dataset(SCIFACT_BASE_URL, SCIFACT_QRELS_BASE_URL, &out)?;
                    println!("{}", serde_json::to_string_pretty(&summary)?);
                }
            },
            EvalCommands::Materialize {
                dataset,
                source,
                out,
            } => match dataset {
                Dataset::Scifact => {
                    let summary = materialize_scifact_dir(&source, &out)?;
                    println!("{}", serde_json::to_string_pretty(&summary)?);
                }
            },
        },
        Commands::Bench { command } => match command {
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
            } => {
                let report = run_quality_benchmark(&QualityBenchmarkRequest {
                    strategy,
                    baseline,
                    command: command_line,
                    corpus_dir: corpus,
                    queries_path: queries,
                    qrels_path: qrels,
                    shortlist,
                    dense_model: DenseModelSpec::with_overrides(
                        model_id,
                        model_revision,
                        max_length,
                    ),
                })?;
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
            } => {
                let report = run_latency_benchmark(&LatencyBenchmarkRequest {
                    strategy,
                    command: command_line,
                    corpus_dir: corpus,
                    queries_path: queries,
                    shortlist,
                    dense_model: DenseModelSpec::with_overrides(
                        model_id,
                        model_revision,
                        max_length,
                    ),
                })?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
        },
        Commands::Search(search) => {
            let (path, query) = search.resolve_targets();
            let response = run_search(&SearchRequest {
                strategy: search.strategy,
                query,
                path,
                limit: search.limit,
                shortlist: search.shortlist,
                dense_model: DenseModelSpec::with_overrides(
                    search.model_id,
                    search.model_revision,
                    search.max_length,
                ),
            })?;
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
