use anyhow::Result;
use std::fmt::Write as _;

use super::super::domain::*;

pub fn render_search_response(response: &SearchResponse, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(response)?),
        OutputFormat::Text => render_text_response(response),
    }
}

pub fn render_autonomous_search_response(
    response: &AutonomousSearchResponse,
    format: OutputFormat,
) -> Result<String> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(response)?),
        OutputFormat::Text => render_autonomous_text_response(response),
    }
}

fn render_text_response(response: &SearchResponse) -> Result<String> {
    let mut output = String::new();

    if response.hits.is_empty() {
        return Ok("no matching hits".to_string());
    }

    for hit in &response.hits {
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

fn render_autonomous_text_response(response: &AutonomousSearchResponse) -> Result<String> {
    let mut output = String::new();
    writeln!(
        &mut output,
        "mode: {}",
        autonomous_search_mode_label(response.mode)
    )?;
    writeln!(
        &mut output,
        "planner: {}",
        planner_strategy_kind_label(response.planner_strategy.kind)
    )?;
    writeln!(&mut output, "turns: {}", response.turns.len())?;
    if let Some(graph_episode) = &response.state.graph_episode {
        writeln!(&mut output, "branches: {}", graph_episode.branches.len())?;
        writeln!(&mut output, "frontier: {}", graph_episode.frontier.len())?;
    }
    if let Some(stop_reason) = response.planner_trace.stop_reason {
        writeln!(
            &mut output,
            "stop: {}",
            autonomous_stop_reason_label(stop_reason)
        )?;
    }

    if let Some(SearchEmission::View(view)) = response
        .turns
        .iter()
        .rev()
        .map(|turn| &turn.emission)
        .find(|emission| matches!(emission, SearchEmission::View(_)))
    {
        let body = render_text_response(view)?;
        if !body.is_empty() {
            writeln!(&mut output)?;
            write!(&mut output, "{body}")?;
        }
    } else if response.turns.is_empty() {
        writeln!(&mut output)?;
        writeln!(&mut output, "no autonomous turns executed")?;
    }

    Ok(output.trim_end().to_string())
}

fn planner_strategy_kind_label(kind: AutonomousPlannerStrategyKind) -> &'static str {
    match kind {
        AutonomousPlannerStrategyKind::Heuristic => "heuristic",
        AutonomousPlannerStrategyKind::ModelDriven => "model-driven",
    }
}

fn autonomous_search_mode_label(mode: AutonomousSearchMode) -> &'static str {
    match mode {
        AutonomousSearchMode::Linear => "linear",
        AutonomousSearchMode::Graph => "graph",
    }
}

fn autonomous_stop_reason_label(reason: AutonomousPlannerStopReason) -> &'static str {
    match reason {
        AutonomousPlannerStopReason::GoalSatisfied => "goal-satisfied",
        AutonomousPlannerStopReason::StepLimitReached => "step-limit-reached",
        AutonomousPlannerStopReason::NoFurtherQueries => "no-further-queries",
        AutonomousPlannerStopReason::NoAdditionalEvidence => "no-additional-evidence",
    }
}
