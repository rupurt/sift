use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use sift::{SearchInput, SearchOptions, SearchResponse, Sift};

#[derive(Parser)]
#[command(name = "sift-embed")]
#[command(about = "Example CLI consuming sift as an embeddable library", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search a local corpus through the supported sift facade
    Search(SearchCommand),
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
#[command(override_usage = "sift-embed search [OPTIONS] [PATH] <QUERY>")]
#[command(after_help = "If PATH is omitted, sift-embed searches the current directory.")]
struct SearchCommand {
    #[arg(long)]
    strategy: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    shortlist: Option<usize>,

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

    fn to_input(&self) -> SearchInput {
        let (path, query) = self.resolve_targets();
        let mut options = SearchOptions::default();

        if let Some(strategy) = &self.strategy {
            options = options.with_strategy(strategy.clone());
        }
        if let Some(limit) = self.limit {
            options = options.with_limit(limit);
        }
        if let Some(shortlist) = self.shortlist {
            options = options.with_shortlist(shortlist);
        }

        SearchInput::new(path, query).with_options(options)
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sift = Sift::builder().build();

    match cli.command {
        Commands::Search(search) => {
            let response = sift.search(search.to_input())?;
            print_response(&response);
        }
    }

    Ok(())
}

fn print_response(response: &SearchResponse) {
    if response.results.is_empty() {
        println!("no matching results");
        return;
    }

    for hit in &response.results {
        println!("{}. {}", hit.rank, hit.path);
        if let Some(location) = &hit.location {
            println!("   location: {location}");
        }
        println!("   score: {:.4} ({:?})", hit.score, hit.confidence);
        if !hit.snippet.is_empty() {
            for (index, line) in hit.snippet.lines().enumerate() {
                if index == 0 {
                    println!("   snippet: {line}");
                } else {
                    println!("            {line}");
                }
            }
        }
        println!();
    }
}
