use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use sift::bench::{
    LatencyBenchmarkRequest, QualityBenchmarkRequest, run_latency_benchmark, run_quality_benchmark,
};
use sift::eval::{download_scifact_dataset, materialize_scifact_dir};
use sift::search::{Engine, OutputFormat, SearchRequest, render_search_response, run_search};

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
    Search {
        #[arg(long, value_enum, default_value_t = Engine::Bm25)]
        engine: Engine,

        #[arg(long)]
        json: bool,

        #[arg(long, default_value_t = 10)]
        limit: usize,

        /// The search query
        query: String,

        /// The path to search
        path: PathBuf,
    },
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
        #[arg(long, value_enum, default_value_t = Engine::Bm25)]
        engine: Engine,
        #[arg(long)]
        corpus: PathBuf,
        #[arg(long)]
        qrels: PathBuf,
    },
    /// Run latency benchmarks
    Latency {
        #[arg(long, value_enum, default_value_t = Engine::Bm25)]
        engine: Engine,
        #[arg(long)]
        corpus: PathBuf,
        #[arg(long)]
        queries: PathBuf,
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
                engine,
                corpus,
                qrels,
            } => {
                let report = run_quality_benchmark(&QualityBenchmarkRequest {
                    engine,
                    command: command_line,
                    corpus_dir: corpus,
                    qrels_path: qrels,
                })?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            BenchCommands::Latency {
                engine,
                corpus,
                queries,
            } => {
                let report = run_latency_benchmark(&LatencyBenchmarkRequest {
                    engine,
                    command: command_line,
                    corpus_dir: corpus,
                    queries_path: queries,
                })?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
        },
        Commands::Search {
            engine,
            json,
            limit,
            query,
            path,
        } => {
            let response = run_search(&SearchRequest {
                engine,
                query,
                path,
                limit,
            })?;
            let output = render_search_response(
                &response,
                if json {
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
