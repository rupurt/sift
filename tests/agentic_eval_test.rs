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
            baseline_strategy: Some("bm25".to_string()),
            planner_strategy: sift::AutonomousPlannerStrategy::heuristic(),
            command: "cargo test --test agentic_eval_test".to_string(),
            corpus_dir: root.join("corpus"),
            fixtures_path: root.join("fixtures.json"),
            shortlist: 2,
            dense_model: DenseModelSpec::default(),
            retained_artifact_limit: 1,
            verbose: 0,
            prompts: None,
        },
        None,
    )
    .expect("run agentic evaluation");

    assert_eq!(report.tasks.len(), 2);
    assert_eq!(report.autonomous.tasks.len(), 2);
    assert_eq!(report.graph.tasks.len(), 2);
    assert_eq!(report.metrics.task_success_rate, 1.0);
    assert_eq!(report.metrics.average_final_recall, 1.0);
    assert_eq!(report.metrics.average_turns, 2.0);
    assert!(
        report.metrics.average_prune_actions >= 1.0,
        "bounded context should force at least one prune action across these fixtures"
    );
    assert_eq!(report.comparison.baseline_strategy, "bm25");
    assert_eq!(
        report
            .comparison
            .planned_controller
            .metrics
            .task_success_rate,
        1.0
    );
    assert_eq!(report.comparison.autonomous.metrics.task_success_rate, 1.0);
    assert_eq!(report.comparison.autonomous.metrics.average_turns, 1.0);
    assert_eq!(report.comparison.graph.metrics.task_success_rate, 1.0);
    assert_eq!(report.comparison.graph.metrics.average_turns, 1.0);
    assert_eq!(
        report.comparison.autonomous.planner_strategy,
        Some(sift::AutonomousPlannerStrategy::heuristic())
    );
    assert_eq!(
        report.comparison.graph.planner_strategy,
        Some(sift::AutonomousPlannerStrategy::heuristic())
    );
    assert_eq!(
        report
            .autonomous
            .metrics
            .average_retained_evidence_efficiency,
        1.0
    );
    assert_eq!(report.graph.mode, sift::AutonomousSearchMode::Graph);
    assert_eq!(
        report
            .graph
            .metrics
            .graph
            .as_ref()
            .expect("graph evaluation metrics")
            .average_frontier_expansion_cost,
        1.0
    );
    assert_eq!(report.autonomous.metrics.stop_reasons.len(), 1);
    assert_eq!(report.graph.metrics.stop_reasons.len(), 1);
    assert_eq!(
        report.comparison.baseline_query_mode,
        "concatenate-planned-turn-queries"
    );
    assert_eq!(
        report.comparison.tasks[0].collapsed_query,
        "alpha module beta module"
    );
    assert_eq!(
        report.comparison.tasks[0].root_task,
        "beta completes final execution path"
    );
    assert_eq!(
        report.comparison.tasks[0].autonomous_final_documents,
        vec!["beta".to_string()]
    );
    assert_eq!(
        report.comparison.tasks[0].graph_final_documents,
        vec!["beta".to_string()]
    );
    assert_eq!(
        report.comparison.tasks[0].planned_controller_final_documents,
        vec!["beta".to_string()]
    );
    assert_eq!(report.comparison.tasks[0].autonomous_turns, 1);
    assert_eq!(report.comparison.tasks[0].graph_turns, 1);
    assert_eq!(report.comparison.tasks[0].planned_controller_turns, 2);
    assert_eq!(
        report.comparison.tasks[0].autonomous_retained_evidence_efficiency,
        1.0
    );
    assert_eq!(
        report.comparison.tasks[0].graph_retained_evidence_efficiency,
        1.0
    );
    assert_eq!(
        report.comparison.tasks[0]
            .graph_metrics
            .as_ref()
            .expect("task graph metrics")
            .frontier_expansion_cost,
        1
    );
    assert_eq!(
        report.comparison.tasks[1].collapsed_query,
        "policy layer audit trail"
    );

    let report_json = serde_json::to_value(&report).expect("serialize agentic report");
    assert_eq!(report_json["tasks"][0]["task_id"], "alpha-beta-handoff");
    assert_eq!(report_json["tasks"][0]["success"], true);
    assert_eq!(report_json["tasks"][0]["final_recall_at_10"], 1.0);
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
    assert_eq!(
        report_json["comparison"]["tasks"][0]["collapsed_query"],
        "alpha module beta module"
    );
    assert_eq!(
        report_json["comparison"]["tasks"][0]["root_task"],
        "beta completes final execution path"
    );
    assert_eq!(
        report_json["comparison"]["tasks"][0]["autonomous_final_documents"][0],
        "beta"
    );
    assert_eq!(
        report_json["comparison"]["tasks"][0]["graph_final_documents"][0],
        "beta"
    );
    assert_eq!(
        report_json["autonomous"]["planner_strategy"]["kind"],
        "heuristic"
    );
    assert_eq!(report_json["graph"]["mode"], "graph");
    assert_eq!(
        report_json["autonomous"]["tasks"][0]["stop_reason"],
        "no-further-queries"
    );
    assert_eq!(
        report_json["graph"]["tasks"][0]["graph"]["frontier_expansion_cost"],
        1
    );
    assert_eq!(
        report_json["comparison"]["autonomous"]["stop_reasons"][0]["tasks"],
        2
    );
    assert_eq!(
        report_json["comparison"]["graph"]["stop_reasons"][0]["tasks"],
        2
    );
    assert_eq!(
        report_json["autonomous"]["metrics"]["task_success_rate"],
        1.0
    );
    assert_eq!(report_json["comparison"]["baseline"]["strategy"], "bm25");
    assert_eq!(
        report_json["comparison"]["autonomous"]["mode"],
        "autonomous-planner"
    );
    assert_eq!(
        report_json["comparison"]["graph"]["mode"],
        "graph-autonomous-planner"
    );
    assert_eq!(
        report_json["comparison"]["planned_controller"]["mode"],
        "planned-controller"
    );
    assert_eq!(
        report_json["comparison"]["delta_graph_vs_autonomous"]["graph"]["average_frontier_expansion_cost"],
        1.0
    );
    assert!(
        report_json["tasks"][0]["trace"]["turns"][1]["decisions"]
            .as_array()
            .expect("turn decisions")
            .iter()
            .any(|decision| decision["action"] == "prune"),
        "trace output should record bounded-context pruning decisions"
    );
}
