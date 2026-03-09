use anyhow::Result;
use std::fmt::Write as _;

use super::super::domain::*;

pub fn render_search_response(response: &SearchResponse, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(response)?),
        OutputFormat::Text => render_text_response(response),
    }
}

fn render_text_response(response: &SearchResponse) -> Result<String> {
    let mut output = String::new();
    writeln!(&mut output, "strategy: {}", response.strategy)?;
    writeln!(&mut output, "root: {}", response.root)?;
    writeln!(&mut output, "indexed_files: {}", response.indexed_files)?;
    writeln!(&mut output, "skipped_files: {}", response.skipped_files)?;

    if response.results.is_empty() {
        writeln!(&mut output, "results: 0")?;
        writeln!(&mut output)?;
        writeln!(&mut output, "no matching results")?;
        return Ok(output.trim_end().to_string());
    }

    writeln!(&mut output, "results: {}", response.results.len())?;
    writeln!(&mut output)?;

    for hit in &response.results {
        writeln!(&mut output, "{}. {}", hit.rank, hit.path)?;
        if let Some(location) = &hit.location {
            writeln!(&mut output, "   location: \x1b[36m{}\x1b[0m", location)?;
        }
        writeln!(&mut output, "   score: {:.4}", hit.score)?;
        if !hit.snippet.is_empty() {
            writeln!(&mut output, "   snippet: {}", hit.snippet)?;
        }
        writeln!(&mut output)?;
    }

    Ok(output.trim_end().to_string())
}
