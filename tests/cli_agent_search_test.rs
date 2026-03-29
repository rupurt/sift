use std::process::Command;

fn sample_corpus() -> tempfile::TempDir {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("alpha.txt"),
        "alpha runtime details for the built in autonomous facade",
    )
    .expect("write alpha corpus file");
    corpus
}

#[test]
fn search_agent_json_exposes_planner_metadata_and_trace() {
    let corpus = sample_corpus();
    let output = Command::new(env!("CARGO_BIN_EXE_sift"))
        .args([
            "search",
            "--strategy",
            "bm25",
            "--json",
            "--agent",
            "find alpha runtime details",
            "--agent-mode",
            "graph",
            corpus.path().to_str().expect("utf8 corpus path"),
        ])
        .output()
        .expect("run sift agent search");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse agent json output");
    assert_eq!(parsed["mode"], "graph");
    assert_eq!(parsed["planner_strategy"]["kind"], "heuristic");
    assert_eq!(
        parsed["planner_trace"]["steps"][0]["decisions"][0]["action"],
        "fork"
    );
    assert_eq!(
        parsed["trace"]["turns"][0]["query"],
        "find alpha runtime details"
    );
    assert_eq!(parsed["turns"][0]["turn"]["turn_id"], "turn-1");
}

#[test]
fn search_agent_text_exposes_planner_summary() {
    let corpus = sample_corpus();
    let output = Command::new(env!("CARGO_BIN_EXE_sift"))
        .args([
            "search",
            "--strategy",
            "bm25",
            "--agent",
            "find alpha runtime details",
            "--agent-mode",
            "graph",
            corpus.path().to_str().expect("utf8 corpus path"),
        ])
        .output()
        .expect("run sift agent search");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("utf8 agent text output");
    assert!(stdout.contains("mode: graph"));
    assert!(stdout.contains("planner: heuristic"));
    assert!(stdout.contains("branches: 2"));
    assert!(stdout.contains("frontier: 0"));
    assert!(stdout.contains("turns: 1"));
    assert!(stdout.contains("stop: no-further-queries"));
    assert!(stdout.contains("alpha.txt"));
}

#[test]
fn search_without_agent_preserves_existing_json_shape() {
    let corpus = sample_corpus();
    let output = Command::new(env!("CARGO_BIN_EXE_sift"))
        .args([
            "search",
            "--strategy",
            "bm25",
            "--json",
            corpus.path().to_str().expect("utf8 corpus path"),
            "alpha",
        ])
        .output()
        .expect("run sift direct search");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse direct search json output");
    assert!(parsed.get("hits").is_some());
    assert!(parsed.get("planner_strategy").is_none());
}
