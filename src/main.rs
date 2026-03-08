use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sift")]
#[command(about = "Standalone hybrid search (BM25 + Vector) for lightning-fast document retrieval", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Index a directory
    Index {
        /// The path to the directory to index
        path: std::path::PathBuf,
    },
    /// Search the index
    Search {
        /// The search query
        query: String,
        
        /// Use hybrid search (Vector + BM25)
        #[arg(short = 'H', long)]
        hybrid: bool,
        
        /// Output search results as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Index { path } => {
            println!("Indexing directory: {:?}", path);
            // TODO: Implement indexing logic
        }
        Commands::Search { query, hybrid, json } => {
            if *json {
                // TODO: Implement JSON search output
                println!("{{ \"query\": \"{}\", \"hybrid\": {}, \"results\": [] }}", query, hybrid);
            } else {
                println!("Searching for: '{}' (hybrid: {})", query, hybrid);
                // TODO: Implement text search output
            }
        }
    }

    Ok(())
}
