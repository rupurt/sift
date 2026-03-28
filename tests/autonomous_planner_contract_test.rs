use sift::{
    AutonomousPlannerState, AutonomousPlannerStepCursor, AutonomousPlannerStrategy,
    AutonomousPlannerStrategyKind, AutonomousSearchRequest, AutonomousSearchResponse, SearchPlan,
    SearchTrace,
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
    let response = AutonomousSearchResponse {
        root_task: "Search Me".to_string(),
        planner_strategy: AutonomousPlannerStrategy::model_driven()
            .with_profile("local-planner-v1"),
        plan: SearchPlan::default_lexical(),
        state: AutonomousPlannerState::new(4)
            .with_current_step(
                AutonomousPlannerStepCursor::new("step-2", 2).with_parent_step_id("step-1"),
            )
            .with_completed(true),
        turns: Vec::new(),
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

    let decoded: AutonomousSearchResponse =
        serde_json::from_value(value).expect("deserialize autonomous response");
    assert_eq!(decoded, response);
}
