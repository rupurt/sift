use sift::{
    AutonomousPlannerAction, AutonomousPlannerDecision, AutonomousPlannerState,
    AutonomousPlannerStepCursor, AutonomousPlannerStopReason, AutonomousPlannerStrategy,
    AutonomousPlannerStrategyKind, AutonomousPlannerTrace, AutonomousPlannerTraceStep,
    AutonomousSearchRequest, AutonomousSearchResponse, SearchPlan, SearchTrace,
};

#[test]
fn autonomous_request_defaults_to_linear_heuristic_planning() {
    let request = AutonomousSearchRequest::new("./docs", "Search Me");

    assert_eq!(request.root_task, "Search Me");
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
        .with_strategy("hybrid")
        .with_planner_strategy(
            AutonomousPlannerStrategy::model_driven().with_profile("local-planner-v1"),
        )
        .with_step_limit(4);

    let value = serde_json::to_value(&request).expect("serialize autonomous request");

    assert_eq!(value["planner_strategy"]["kind"], "model-driven");
    assert_eq!(value["planner_strategy"]["profile"], "local-planner-v1");
    assert_eq!(value["state"]["current_step"]["step_id"], "step-1");
    assert_eq!(value["state"]["step_limit"], 4);

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
        planner_strategy: planner_strategy.clone(),
        plan: SearchPlan::default_lexical(),
        state: AutonomousPlannerState::new(4)
            .with_current_step(
                AutonomousPlannerStepCursor::new("step-2", 2).with_parent_step_id("step-1"),
            )
            .with_completed(true),
        turns: Vec::new(),
        planner_trace: AutonomousPlannerTrace::new(planner_strategy)
            .with_session_id("session-1")
            .with_steps(vec![
                AutonomousPlannerTraceStep::new(AutonomousPlannerStepCursor::new("step-1", 1))
                    .with_decisions(vec![
                        AutonomousPlannerDecision::new(AutonomousPlannerAction::Search)
                            .with_query("cache invalidation path")
                            .with_turn_id("turn-1")
                            .with_rationale("start with the explicit subsystem name"),
                        AutonomousPlannerDecision::new(AutonomousPlannerAction::Continue)
                            .with_next_step(
                                AutonomousPlannerStepCursor::new("step-2", 2)
                                    .with_parent_step_id("step-1"),
                            )
                            .with_rationale("follow the evidence into the next step"),
                    ]),
                AutonomousPlannerTraceStep::new(
                    AutonomousPlannerStepCursor::new("step-2", 2).with_parent_step_id("step-1"),
                )
                .with_decisions(vec![
                    AutonomousPlannerDecision::new(AutonomousPlannerAction::Terminate)
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

    assert_eq!(value["state"]["current_step"]["parent_step_id"], "step-1");
    assert_eq!(value["state"]["completed"], true);
    assert_eq!(
        value["planner_trace"]["steps"][0]["decisions"][0]["action"],
        "search"
    );
    assert_eq!(
        value["planner_trace"]["steps"][1]["decisions"][0]["stop_reason"],
        "goal-satisfied"
    );

    let decoded: AutonomousSearchResponse =
        serde_json::from_value(value).expect("deserialize autonomous response");
    assert_eq!(decoded, response);
}
