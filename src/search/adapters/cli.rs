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

    if response.results.is_empty() {
        return Ok("no matching results".to_string());
    }

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
            for (i, line) in hit.snippet.lines().enumerate() {
                if i == 0 {
                    writeln!(&mut output, "   snippet: {}", line)?;
                } else {
                    writeln!(&mut output, "            {}", line)?;
                }
            }
        }
        writeln!(&mut output)?;
    }

    Ok(output.trim_end().to_string())
}
