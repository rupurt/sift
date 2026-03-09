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
    let mut root = response.root.clone();
    if root.starts_with("./") {
        root = root.chars().skip(2).collect();
    }
    writeln!(&mut output, "root: {}", root)?;
    writeln!(&mut output, "indexed_files: {}", response.indexed_files)?;
    writeln!(&mut output, "skipped_files: {}", response.skipped_files)?;

    if response.results.is_empty() {
        writeln!(&mut output, "results: 0")?;
        writeln!(&mut output)?;
        writeln!(&mut output, "no matching results")?;
        return Ok(output.trim_end().to_string());
    }

    writeln!(&mut output, "results: {}", response.results.len())?;
    if let Some(t) = &response.telemetry {
        writeln!(
            &mut output,
            "cache_hits: heuristic={:.1}%, blob={:.1}%, embedding={:.1}%",
            t.heuristic_hit_rate * 100.0,
            t.blob_hit_rate * 100.0,
            t.embedding_hit_rate * 100.0
        )?;
    }
    writeln!(&mut output)?;

    for hit in &response.results {
        writeln!(&mut output, "{}. \x1b[1;32m{}\x1b[0m", hit.rank, hit.path)?;
        if let Some(location) = &hit.location {
            writeln!(&mut output, "   location: \x1b[36m{}\x1b[0m", location)?;
        }
        let score_color = match hit.confidence {
            ScoreConfidence::High => "\x1b[1;32m",   // Bold Green
            ScoreConfidence::Medium => "\x1b[1;33m", // Bold Yellow
            ScoreConfidence::Low => "\x1b[1;31m",    // Bold Red
        };
        writeln!(
            &mut output,
            "   score: {}{:.4}\x1b[0m ({:?})",
            score_color, hit.score, hit.confidence
        )?;
        if !hit.snippet.is_empty() {
            writeln!(&mut output, "   snippet: {}", hit.snippet)?;
        }
        writeln!(&mut output)?;
    }

    Ok(output.trim_end().to_string())
}
