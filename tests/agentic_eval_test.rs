use std::path::PathBuf;

use sift::internal::{
    dense::DenseModelSpec,
    eval::{AgenticEvaluationRequest, run_agentic_evaluation},
};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("agentic-eval")
}

#[test]
fn agentic_harness_runs_planned_multi_turn_fixtures_from_repo_local_files() {
    let root = fixture_root();
    let report = run_agentic_evaluation(
        &AgenticEvaluationRequest {
            strategy: "bm25".to_string(),
            command: "cargo test --test agentic_eval_test".to_string(),
            corpus_dir: root.join("corpus"),
            fixtures_path: root.join("fixtures.json"),
            shortlist: 2,
            dense_model: DenseModelSpec::default(),
            retained_evidence_limit: 1,
            verbose: 0,
            prompts: None,
        },
        None,
    )
    .expect("run agentic evaluation");

    assert_eq!(report.tasks.len(), 2);
    assert_eq!(report.metrics.task_success_rate, 1.0);
    assert_eq!(report.metrics.average_turns, 2.0);
    assert!(
        report.metrics.average_prune_actions >= 1.0,
        "bounded context should force at least one prune action across these fixtures"
    );

    let report_json = serde_json::to_value(&report).expect("serialize agentic report");
    assert_eq!(report_json["tasks"][0]["task_id"], "alpha-beta-handoff");
    assert_eq!(report_json["tasks"][0]["success"], true);
    assert_eq!(
        report_json["tasks"][0]["turns"][0]["expected_documents"][0],
        "alpha"
    );
    assert_eq!(
        report_json["tasks"][0]["turns"][1]["expected_documents"][0],
        "beta"
    );
    assert_eq!(
        report_json["tasks"][1]["task_id"],
        "policy-audit-escalation"
    );
    assert_eq!(report_json["tasks"][1]["success"], true);
    assert!(
        report_json["tasks"][0]["trace"]["turns"][1]["decisions"]
            .as_array()
            .expect("turn decisions")
            .iter()
            .any(|decision| decision["action"] == "prune"),
        "trace output should record bounded-context pruning decisions"
    );
}
