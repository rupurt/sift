use sift::{
    AutonomousGraphBranchState, AutonomousGraphBranchStatus, AutonomousGraphEdge,
    AutonomousGraphEdgeKind, AutonomousGraphEpisodeState, AutonomousGraphFrontierEntry,
    AutonomousGraphNode, AutonomousPlannerAction, AutonomousPlannerDecision,
    AutonomousPlannerState, AutonomousPlannerStepCursor, AutonomousPlannerStopReason,
    AutonomousPlannerStrategy, AutonomousPlannerStrategyKind, AutonomousPlannerTrace,
    AutonomousPlannerTraceStep, AutonomousSearchMode, AutonomousSearchRequest,
    AutonomousSearchResponse, SearchPlan, SearchTrace,
};

#[test]
fn autonomous_request_defaults_to_linear_heuristic_planning() {
    let request = AutonomousSearchRequest::new("./docs", "Search Me");

    assert_eq!(request.root_task, "Search Me");
    assert_eq!(request.mode, AutonomousSearchMode::Linear);
    assert_eq!(
        request.planner_strategy.kind,
        AutonomousPlannerStrategyKind::Heuristic
    );
    assert_eq!(request.state.current_step.step_id, "step-1");
    assert_eq!(request.state.current_step.sequence, 1);
    assert_eq!(request.state.step_limit, 3);
    assert!(request.state.retained_artifacts.is_empty());
    assert!(!request.state.completed);
    assert_eq!(request.retained_artifact_limit, 5);
}

#[test]
fn autonomous_request_round_trips_through_serde() {
    let request = AutonomousSearchRequest::new("./docs", "Search Me")
        .with_session_id("session-1")
        .with_mode(AutonomousSearchMode::Graph)
        .with_strategy("hybrid")
        .with_planner_strategy(
            AutonomousPlannerStrategy::model_driven().with_profile("local-planner-v1"),
        )
        .with_step_limit(4)
        .with_graph_episode(
            AutonomousGraphEpisodeState::new()
                .with_root_node_id("node-root")
                .with_active_branch_id("branch-root")
                .with_nodes(vec![AutonomousGraphNode::new(
                    "node-root",
                    "branch-root",
                    AutonomousPlannerStepCursor::first(),
                )])
                .with_branches(vec![
                    AutonomousGraphBranchState::new("branch-root", "node-root")
                        .with_status(AutonomousGraphBranchStatus::Active),
                ])
                .with_frontier(vec![AutonomousGraphFrontierEntry::new(
                    "frontier-root",
                    "branch-root",
                    "node-root",
                )]),
        );

    let value = serde_json::to_value(&request).expect("serialize autonomous request");

    assert_eq!(value["mode"], "graph");
    assert_eq!(value["planner_strategy"]["kind"], "model-driven");
    assert_eq!(value["planner_strategy"]["profile"], "local-planner-v1");
    assert_eq!(value["state"]["current_step"]["step_id"], "step-1");
    assert_eq!(value["state"]["step_limit"], 4);
    assert_eq!(value["state"]["graph_episode"]["root_node_id"], "node-root");

    let decoded: AutonomousSearchRequest =
        serde_json::from_value(value).expect("deserialize autonomous request");
    assert_eq!(decoded, request);
}

#[test]
fn autonomous_response_round_trips_through_serde() {
    let planner_strategy =
        AutonomousPlannerStrategy::model_driven().with_profile("local-planner-v1");
    let response = AutonomousSearchResponse {
        root_task: "Search Me".to_string(),
        mode: AutonomousSearchMode::Graph,
        planner_strategy: planner_strategy.clone(),
        plan: SearchPlan::default_lexical(),
        state: AutonomousPlannerState::new(4)
            .with_current_step(
                AutonomousPlannerStepCursor::new("step-2", 2).with_parent_step_id("step-1"),
            )
            .with_graph_episode(
                AutonomousGraphEpisodeState::new()
                    .with_root_node_id("node-root")
                    .with_active_branch_id("branch-a")
                    .with_nodes(vec![
                        AutonomousGraphNode::new(
                            "node-root",
                            "branch-root",
                            AutonomousPlannerStepCursor::first(),
                        ),
                        AutonomousGraphNode::new(
                            "node-a",
                            "branch-a",
                            AutonomousPlannerStepCursor::new("node-a", 2)
                                .with_parent_step_id("step-1"),
                        )
                        .with_query("cache invalidation path"),
                    ])
                    .with_edges(vec![AutonomousGraphEdge::new(
                        "edge-root-a",
                        "node-root",
                        "node-a",
                        AutonomousGraphEdgeKind::Child,
                    )])
                    .with_branches(vec![
                        AutonomousGraphBranchState::new("branch-root", "node-root")
                            .with_status(AutonomousGraphBranchStatus::Completed),
                        AutonomousGraphBranchState::new("branch-a", "node-a")
                            .with_status(AutonomousGraphBranchStatus::Completed),
                    ])
                    .with_completed(true),
            )
            .with_completed(true),
        turns: Vec::new(),
        planner_trace: AutonomousPlannerTrace::new(planner_strategy)
            .with_session_id("session-1")
            .with_steps(vec![
                AutonomousPlannerTraceStep::new(AutonomousPlannerStepCursor::new("step-1", 1))
                    .with_decisions(vec![
                        AutonomousPlannerDecision::new(AutonomousPlannerAction::Fork)
                            .with_branch_id("branch-root")
                            .with_node_id("node-root")
                            .with_target_branch_id("branch-a")
                            .with_target_node_id("node-a")
                            .with_edge_id("edge-root-a")
                            .with_edge_kind(AutonomousGraphEdgeKind::Child)
                            .with_frontier_id("frontier-a")
                            .with_query("cache invalidation path")
                            .with_next_step(
                                AutonomousPlannerStepCursor::new("node-a", 2)
                                    .with_parent_step_id("step-1"),
                            )
                            .with_rationale("fork a child branch from the root node"),
                        AutonomousPlannerDecision::new(AutonomousPlannerAction::Select)
                            .with_branch_id("branch-a")
                            .with_node_id("node-a")
                            .with_frontier_id("frontier-a")
                            .with_rationale("activate the forked frontier branch"),
                        AutonomousPlannerDecision::new(AutonomousPlannerAction::Search)
                            .with_branch_id("branch-a")
                            .with_node_id("node-a")
                            .with_query("cache invalidation path")
                            .with_turn_id("turn-1")
                            .with_rationale("start with the explicit subsystem name"),
                    ]),
                AutonomousPlannerTraceStep::new(
                    AutonomousPlannerStepCursor::new("step-2", 2).with_parent_step_id("step-1"),
                )
                .with_decisions(vec![
                    AutonomousPlannerDecision::new(AutonomousPlannerAction::Terminate)
                        .with_branch_id("branch-a")
                        .with_stop_reason(AutonomousPlannerStopReason::GoalSatisfied)
                        .with_rationale("the retained evidence answers the root task"),
                ]),
            ])
            .with_completed(true)
            .with_stop_reason(AutonomousPlannerStopReason::GoalSatisfied),
        trace: SearchTrace {
            session_id: Some("session-1".to_string()),
            turns: Vec::new(),
            completed: true,
            termination_reason: None,
        },
    };

    let value = serde_json::to_value(&response).expect("serialize autonomous response");

    assert_eq!(value["mode"], "graph");
    assert_eq!(value["state"]["current_step"]["parent_step_id"], "step-1");
    assert_eq!(value["state"]["completed"], true);
    assert_eq!(value["state"]["graph_episode"]["edges"][0]["kind"], "child");
    assert_eq!(
        value["planner_trace"]["steps"][0]["decisions"][0]["action"],
        "fork"
    );
    assert_eq!(
        value["planner_trace"]["steps"][1]["decisions"][0]["stop_reason"],
        "goal-satisfied"
    );

    let decoded: AutonomousSearchResponse =
        serde_json::from_value(value).expect("deserialize autonomous response");
    assert_eq!(decoded, response);
}
