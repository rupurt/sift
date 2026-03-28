use sift::{
    AcquisitionAdapterKind, ArtifactBudget, ArtifactFreshness, ArtifactProvenance,
    AutonomousPlanner, AutonomousPlannerAction, AutonomousPlannerState,
    AutonomousPlannerStopReason, AutonomousPlannerStrategy, AutonomousPlannerStrategyKind,
    AutonomousSearchRequest, ContextArtifactKind, EnvironmentFactInput, HeuristicAutonomousPlanner,
    LocalContextSource, RetainedArtifact, ToolOutputInput,
};

fn retained_artifact(
    artifact_id: &str,
    path: &str,
    snippet: &str,
    rationale: &str,
) -> RetainedArtifact {
    RetainedArtifact::new(
        artifact_id,
        ContextArtifactKind::File,
        path,
        ArtifactProvenance {
            adapter: AcquisitionAdapterKind::FileSystem,
            source: "test".to_string(),
            synthetic: false,
        },
        ArtifactFreshness {
            observed_unix_secs: 1,
            modified_unix_secs: Some(1),
        },
        ArtifactBudget::from_text(snippet, 1),
    )
    .with_snippet(snippet)
    .with_rationale(rationale)
}

#[test]
fn heuristic_planner_is_public_and_emits_initial_query_from_root_task_and_context() {
    let planner = HeuristicAutonomousPlanner::default();
    let request = AutonomousSearchRequest::new("./docs", "find cache invalidation")
        .with_local_context(vec![
            LocalContextSource::EnvironmentFact(EnvironmentFactInput::new("cwd", "src/cache")),
            LocalContextSource::ToolOutput(ToolOutputInput::new(
                "rg",
                "call-1",
                "retry loop adapter",
            )),
        ]);

    let trace = planner.plan(&request).expect("heuristic planner trace");

    assert_eq!(
        trace.planner_strategy.kind,
        AutonomousPlannerStrategyKind::Heuristic
    );
    assert_eq!(trace.steps.len(), 1);
    assert!(trace.completed);
    assert_eq!(
        trace.stop_reason,
        Some(AutonomousPlannerStopReason::NoFurtherQueries)
    );
    assert_eq!(
        trace.steps[0].decisions[0].query.as_deref(),
        Some("find cache invalidation retry loop adapter")
    );
    assert_eq!(
        trace.steps[0].decisions[1].action,
        AutonomousPlannerAction::Terminate
    );
}

#[test]
fn heuristic_planner_deduplicates_follow_up_queries_from_retained_evidence() {
    let planner = HeuristicAutonomousPlanner::default();
    let request = AutonomousSearchRequest::new("./docs", "cache invalidation path").with_state(
        AutonomousPlannerState::new(3).with_retained_artifacts(vec![
            retained_artifact(
                "artifact-1",
                "src/cache.rs",
                "cache invalidation retry loop adapter layer",
                "follow the retry loop implementation",
            ),
            retained_artifact(
                "artifact-2",
                "src/cache.rs",
                "cache invalidation retry loop adapter layer",
                "follow the retry loop implementation",
            ),
            retained_artifact(
                "artifact-3",
                "src/cache_state.rs",
                "planner cursor evidence persistence",
                "inspect retained evidence persistence",
            ),
        ]),
    );

    let trace = planner.plan(&request).expect("heuristic planner trace");

    assert_eq!(trace.steps.len(), 3);
    assert_eq!(
        trace.steps[0].decisions[0].query.as_deref(),
        Some("cache invalidation path")
    );
    assert_eq!(
        trace.steps[1].decisions[0].query.as_deref(),
        Some("retry loop adapter layer")
    );
    assert_eq!(
        trace.steps[2].decisions[0].query.as_deref(),
        Some("planner cursor evidence persistence state")
    );
    assert_eq!(
        trace.steps[0].decisions[1].action,
        AutonomousPlannerAction::Continue
    );
    assert_eq!(
        trace.steps[1].decisions[1].action,
        AutonomousPlannerAction::Continue
    );
    assert_eq!(
        trace.steps[2].decisions[1].stop_reason,
        Some(AutonomousPlannerStopReason::NoAdditionalEvidence)
    );
    assert!(trace.completed);
    assert_eq!(
        trace.stop_reason,
        Some(AutonomousPlannerStopReason::NoAdditionalEvidence)
    );
}

#[test]
fn heuristic_planner_emits_step_limit_stop_reason_when_queries_are_truncated() {
    let planner = HeuristicAutonomousPlanner::default();
    let request = AutonomousSearchRequest::new("./docs", "cache invalidation path").with_state(
        AutonomousPlannerState::new(2).with_retained_artifacts(vec![
            retained_artifact(
                "artifact-1",
                "src/cache.rs",
                "cache invalidation retry loop adapter layer",
                "follow the retry loop implementation",
            ),
            retained_artifact(
                "artifact-2",
                "src/cache_state.rs",
                "planner cursor evidence persistence",
                "inspect retained evidence persistence",
            ),
        ]),
    );

    let trace = planner.plan(&request).expect("heuristic planner trace");

    assert_eq!(trace.steps.len(), 2);
    assert_eq!(
        trace.steps[1].decisions[1].stop_reason,
        Some(AutonomousPlannerStopReason::StepLimitReached)
    );
    assert_eq!(
        trace.stop_reason,
        Some(AutonomousPlannerStopReason::StepLimitReached)
    );
}

#[test]
fn heuristic_planner_is_deterministic_for_same_request_and_evidence() {
    let planner = HeuristicAutonomousPlanner::default();
    let request = AutonomousSearchRequest::new("./docs", "cache invalidation path").with_state(
        AutonomousPlannerState::new(3).with_retained_artifacts(vec![
            retained_artifact(
                "artifact-1",
                "src/cache.rs",
                "cache invalidation retry loop adapter layer",
                "follow the retry loop implementation",
            ),
            retained_artifact(
                "artifact-2",
                "src/cache_state.rs",
                "planner cursor evidence persistence",
                "inspect retained evidence persistence",
            ),
        ]),
    );

    let first = planner
        .plan(&request)
        .expect("first heuristic planner trace");
    let second = planner
        .plan(&request)
        .expect("second heuristic planner trace");

    assert_eq!(first, second);
}

#[test]
fn heuristic_planner_rejects_model_driven_requests() {
    let planner = HeuristicAutonomousPlanner::default();
    let request = AutonomousSearchRequest::new("./docs", "Search Me").with_planner_strategy(
        AutonomousPlannerStrategy::model_driven().with_profile("local-planner-v1"),
    );

    let error = planner
        .plan(&request)
        .expect_err("strategy mismatch should fail");

    assert!(
        error
            .to_string()
            .contains("heuristic planner requires the heuristic planner strategy")
    );
}
