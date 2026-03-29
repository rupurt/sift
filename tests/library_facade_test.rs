use sift::{
    AcquisitionAdapterKind, AgentTurnInput, ArtifactBudget, ArtifactFreshness, ArtifactProvenance,
    AutonomousPlanner, AutonomousPlannerAction, AutonomousPlannerDecision,
    AutonomousPlannerStopReason, AutonomousPlannerStrategy, AutonomousPlannerTrace,
    AutonomousPlannerTraceStep, AutonomousSearchMode, AutonomousSearchRequest, ContextArtifactKind,
    ContextAssemblyBudget, ContextAssemblyRequest, Conversation, EnvironmentFactInput, Fusion,
    FusionPolicy, GenerativeModel, QueryExpansionPolicy, Reranking, RerankingPolicy,
    RetainedArtifact, Retriever, RetrieverPolicy, SearchControllerAction, SearchControllerRequest,
    SearchEmission, SearchEmissionMode, SearchInput, SearchOptions, SearchPlan, SearchTurnRequest,
    Sift, ToolOutputInput,
};

struct SingleStepPlanner;
struct InvalidGraphPlanner;

struct EmptyConversation;

impl Conversation for EmptyConversation {
    fn send(&mut self, _message: &str, _max_tokens: usize) -> anyhow::Result<String> {
        Ok(String::new())
    }

    fn history(&self) -> &[String] {
        &[]
    }
}

struct StaticGenerativeModel {
    output: String,
}

impl GenerativeModel for StaticGenerativeModel {
    fn generate(&self, _prompt: &str, _max_tokens: usize) -> anyhow::Result<String> {
        Ok(self.output.clone())
    }

    fn start_conversation(&self) -> anyhow::Result<Box<dyn Conversation>> {
        Ok(Box::new(EmptyConversation))
    }
}

impl AutonomousPlanner for SingleStepPlanner {
    fn plan(&self, request: &AutonomousSearchRequest) -> anyhow::Result<AutonomousPlannerTrace> {
        Ok(
            AutonomousPlannerTrace::new(request.planner_strategy.clone())
                .with_steps(vec![
                    AutonomousPlannerTraceStep::new(sift::AutonomousPlannerStepCursor::new(
                        "step-1", 1,
                    ))
                    .with_decisions(vec![
                        AutonomousPlannerDecision::new(AutonomousPlannerAction::Search)
                            .with_query("alpha")
                            .with_turn_id("turn-a")
                            .with_rationale("start with the most explicit token"),
                        AutonomousPlannerDecision::new(AutonomousPlannerAction::Terminate)
                            .with_stop_reason(AutonomousPlannerStopReason::GoalSatisfied)
                            .with_rationale("the root task is satisfied by the first turn"),
                    ]),
                ])
                .with_completed(true)
                .with_stop_reason(AutonomousPlannerStopReason::GoalSatisfied),
        )
    }
}

impl AutonomousPlanner for InvalidGraphPlanner {
    fn plan(&self, request: &AutonomousSearchRequest) -> anyhow::Result<AutonomousPlannerTrace> {
        Ok(
            AutonomousPlannerTrace::new(request.planner_strategy.clone())
                .with_steps(vec![AutonomousPlannerTraceStep::new(
                    sift::AutonomousPlannerStepCursor::new("step-1", 1),
                )
                .with_decisions(vec![
                    AutonomousPlannerDecision::new(AutonomousPlannerAction::Select)
                        .with_branch_id("branch-root")
                        .with_node_id("step-1")
                        .with_frontier_id("frontier-missing")
                        .with_rationale("select an impossible frontier"),
                ])])
                .with_completed(true)
                .with_stop_reason(AutonomousPlannerStopReason::NoFurtherQueries),
        )
    }
}

fn custom_lexical_plan(name: &str) -> SearchPlan {
    SearchPlan {
        name: name.to_string(),
        query_expansion: QueryExpansionPolicy::None,
        retrievers: vec![RetrieverPolicy::Bm25],
        fusion: FusionPolicy::Rrf,
        reranking: RerankingPolicy::None,
    }
}

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
fn embedded_facade_runs_a_basic_bm25_search() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("guide.txt"),
        "canonical facade search integration",
    )
    .expect("write corpus file");

    let engine = Sift::builder().build();
    let response = engine
        .search(
            SearchInput::new(corpus.path(), "canonical").with_options(
                SearchOptions::default()
                    .with_strategy("bm25")
                    .with_retrievers(vec![Retriever::Bm25])
                    .with_fusion(Fusion::Rrf)
                    .with_reranking(Reranking::None)
                    .with_limit(1)
                    .with_shortlist(1),
            ),
        )
        .expect("search through facade");

    assert_eq!(response.hits.len(), 1);
    assert!(response.hits[0].path.ends_with("guide.txt"));
}

#[test]
fn embedded_facade_searches_environment_tool_and_turn_context_as_artifacts() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(corpus.path().join("guide.txt"), "filesystem guide content")
        .expect("write corpus file");

    let engine = Sift::builder().build();
    let response = engine
        .search(SearchInput::new(corpus.path(), "telemetry").with_options(
            SearchOptions::default().with_local_context(vec![
                    sift::LocalContextSource::EnvironmentFact(EnvironmentFactInput::new(
                        "cwd",
                        "/repo",
                    )),
                    sift::LocalContextSource::ToolOutput(ToolOutputInput::new(
                        "rg",
                        "call-1",
                        "telemetry span waterfall",
                    )),
                    sift::LocalContextSource::AgentTurn(
                        AgentTurnInput::new("turn-1", "assistant", "retain the telemetry result")
                            .with_session_id("session-ctx"),
                    ),
                ]),
        ))
        .expect("search with synthetic local context");

    assert_eq!(response.indexed_artifacts, 4);
    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.artifact_kind == ContextArtifactKind::ToolOutput
                && hit.provenance.adapter == AcquisitionAdapterKind::ToolOutput
                && hit.provenance.synthetic)
    );
}

#[test]
fn embedded_facade_projects_search_into_protocol_turns() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("guide.txt"),
        "canonical facade search integration",
    )
    .expect("write corpus file");

    let engine = Sift::builder().build();
    let response = engine
        .search_turn(
            SearchTurnRequest::new(corpus.path(), "canonical")
                .with_session_id("session-1")
                .with_turn_id("turn-1")
                .with_strategy("bm25")
                .with_limit(1)
                .with_shortlist(1)
                .with_emission_mode(SearchEmissionMode::Protocol),
        )
        .expect("turn-aware search");

    assert_eq!(response.turn.turn_id, "turn-1");
    assert_eq!(response.turn.sequence, 1);
    assert_eq!(response.turn.result_count, 1);
    assert_eq!(response.assembly.response.hits.len(), 1);
    assert_eq!(response.assembly.pruned_artifacts, 0);
    assert_eq!(response.trace.turns.len(), 1);
    assert!(response.trace.completed);
    assert_eq!(
        response.trace.turns[0]
            .decisions
            .last()
            .expect("terminal decision")
            .action,
        SearchControllerAction::Terminate
    );

    match response.emission {
        SearchEmission::Protocol(protocol) => {
            assert_eq!(protocol.turn_id, "turn-1");
            assert_eq!(protocol.hits.len(), 1);
            assert!(protocol.hits[0].path.ends_with("guide.txt"));
        }
        other => panic!("expected protocol emission, got {:?}", other),
    }
}

#[test]
fn embedded_facade_exposes_context_assembly_surface() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("guide.txt"),
        "canonical assembly surface",
    )
    .expect("write corpus file");

    let engine = Sift::builder().build();
    let response = engine
        .assemble_context(
            ContextAssemblyRequest::new(corpus.path(), "telemetry")
                .with_strategy("bm25")
                .with_limit(2)
                .with_budget(ContextAssemblyBudget::new(1))
                .with_local_context(vec![sift::LocalContextSource::ToolOutput(
                    ToolOutputInput::new("rg", "call-assembly", "telemetry assembly contract"),
                )])
                .with_emission_mode(SearchEmissionMode::Protocol),
        )
        .expect("context assembly");

    assert_eq!(response.response.indexed_artifacts, 2);
    assert_eq!(response.response.hits.len(), 1);
    assert_eq!(response.retained_artifacts.len(), 1);
    assert_eq!(
        response.retained_artifacts[0].artifact_kind,
        ContextArtifactKind::ToolOutput
    );

    match response.emission {
        SearchEmission::Protocol(protocol) => {
            assert_eq!(protocol.turn_id, "assembly");
            assert_eq!(protocol.hits.len(), 1);
            assert_eq!(
                protocol.hits[0].artifact_kind,
                ContextArtifactKind::ToolOutput
            );
        }
        other => panic!("expected protocol emission, got {:?}", other),
    }
}

#[test]
fn turn_search_emits_synthetic_agent_turn_hits_through_protocol_mode() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(corpus.path().join("guide.txt"), "filesystem guide content")
        .expect("write corpus file");

    let engine = Sift::builder().build();
    let response = engine
        .search_turn(
            SearchTurnRequest::new(corpus.path(), "refactor")
                .with_turn_id("turn-local-context")
                .with_strategy("bm25")
                .with_limit(1)
                .with_shortlist(1)
                .with_local_context(vec![sift::LocalContextSource::AgentTurn(
                    AgentTurnInput::new("turn-1", "assistant", "refactor the adapter layer")
                        .with_session_id("session-local"),
                )])
                .with_emission_mode(SearchEmissionMode::Protocol),
        )
        .expect("protocol turn search with local context");

    match response.emission {
        SearchEmission::Protocol(protocol) => {
            assert_eq!(protocol.hits.len(), 1);
            assert_eq!(
                protocol.hits[0].artifact_kind,
                ContextArtifactKind::AgentTurn
            );
            assert_eq!(
                protocol.hits[0].provenance.adapter,
                AcquisitionAdapterKind::AgentTurn
            );
            assert!(protocol.hits[0].path.contains(".sift/context/agent-turn"));
        }
        other => panic!("expected protocol emission, got {:?}", other),
    }
}

#[test]
fn embedded_facade_projects_search_into_latent_emissions() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("guide.txt"),
        "canonical facade search integration",
    )
    .expect("write corpus file");

    let engine = Sift::builder().build();
    let response = engine
        .search_turn(
            SearchTurnRequest::new(corpus.path(), "canonical")
                .with_turn_id("turn-2")
                .with_strategy("bm25")
                .with_limit(1)
                .with_shortlist(1)
                .with_emission_mode(SearchEmissionMode::Latent),
        )
        .expect("latent emission search");

    match response.emission {
        SearchEmission::Latent(latent) => {
            assert_eq!(latent.turn_id, "turn-2");
            assert_eq!(latent.feature_space, "ranking-score");
            assert_eq!(latent.hits.len(), 1);
            assert!(latent.hits[0].path.ends_with("guide.txt"));
        }
        other => panic!("expected latent emission, got {:?}", other),
    }
}

#[test]
fn emission_contracts_round_trip_through_serde() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("guide.txt"),
        "serde-stable emission contracts remain inspectable",
    )
    .expect("write corpus file");

    let engine = Sift::builder().build();
    let protocol = engine
        .search_turn(
            SearchTurnRequest::new(corpus.path(), "serde-stable")
                .with_session_id("session-serde")
                .with_turn_id("turn-protocol")
                .with_strategy("bm25")
                .with_limit(1)
                .with_shortlist(1)
                .with_emission_mode(SearchEmissionMode::Protocol),
        )
        .expect("protocol turn search");
    let latent = engine
        .search_turn(
            SearchTurnRequest::new(corpus.path(), "serde-stable")
                .with_session_id("session-serde")
                .with_turn_id("turn-latent")
                .with_strategy("bm25")
                .with_limit(1)
                .with_shortlist(1)
                .with_emission_mode(SearchEmissionMode::Latent),
        )
        .expect("latent turn search");

    let protocol_json =
        serde_json::to_value(&protocol).expect("serialize protocol emission contract");
    let latent_json = serde_json::to_value(&latent).expect("serialize latent emission contract");

    assert_eq!(protocol_json["emission"]["kind"], "protocol");
    assert_eq!(
        protocol_json["trace"]["turns"][0]["emission_mode"],
        "protocol"
    );
    assert_eq!(latent_json["emission"]["kind"], "latent");
    assert_eq!(latent_json["trace"]["turns"][0]["emission_mode"], "latent");

    let decoded_protocol: sift::SearchTurnResponse =
        serde_json::from_value(protocol_json).expect("decode protocol response");
    let decoded_latent: sift::SearchTurnResponse =
        serde_json::from_value(latent_json).expect("decode latent response");

    assert_eq!(decoded_protocol, protocol);
    assert_eq!(decoded_latent, latent);
}

#[test]
fn embedded_facade_accepts_explicit_turn_plans_without_registry_lookup() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("guide.txt"),
        "explicit plans should execute without preset lookup",
    )
    .expect("write corpus file");

    let plan = custom_lexical_plan("custom-lexical");
    let engine = Sift::builder().build();
    let response = engine
        .search_turn(
            SearchTurnRequest::new(corpus.path(), "preset lookup")
                .with_strategy("missing-preset")
                .with_plan(plan.clone())
                .with_limit(1)
                .with_shortlist(1),
        )
        .expect("turn search with explicit plan");

    assert_eq!(response.turn.strategy, plan.name);
    assert_eq!(response.turn.result_count, 1);

    match response.emission {
        SearchEmission::View(view) => {
            assert_eq!(view.strategy, "custom-lexical");
            assert!(view.hits[0].path.ends_with("guide.txt"));
        }
        other => panic!("expected view emission, got {:?}", other),
    }
}

#[test]
fn embedded_facade_runs_controller_turns_from_explicit_plan_state() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("alpha.txt"),
        "alpha implementation details for the controller loop",
    )
    .expect("write alpha corpus file");
    std::fs::write(
        corpus.path().join("beta.txt"),
        "beta implementation details for the controller loop",
    )
    .expect("write beta corpus file");

    let plan = custom_lexical_plan("controller-lexical");
    let engine = Sift::builder().build();
    let response = engine
        .search_controller(
            SearchControllerRequest::new(
                plan.clone(),
                vec![
                    SearchTurnRequest::new(corpus.path(), "alpha")
                        .with_strategy("missing-preset")
                        .with_turn_id("turn-a")
                        .with_limit(1)
                        .with_shortlist(1),
                    SearchTurnRequest::new(corpus.path(), "beta")
                        .with_strategy("missing-preset")
                        .with_turn_id("turn-b")
                        .with_limit(1)
                        .with_shortlist(1),
                ],
            )
            .with_session_id("session-1")
            .with_retained_artifact_limit(2),
        )
        .expect("controller search with explicit plan");

    assert_eq!(response.plan.name, "controller-lexical");
    assert_eq!(response.turns.len(), 2);
    assert_eq!(response.trace.turns.len(), 2);
    assert!(response.state.completed);
    assert_eq!(response.state.next_turn, 2);
    assert!(response.state.retained_artifacts.len() <= 2);
    assert_eq!(response.turns[0].turn.strategy, "controller-lexical");
    assert_eq!(response.turns[1].turn.strategy, "controller-lexical");
    assert!(!response.turns[1].turn.retained_artifacts.is_empty());
    assert_eq!(
        response.trace.turns[0]
            .decisions
            .last()
            .expect("first turn continuation")
            .action,
        SearchControllerAction::Continue
    );
    assert_eq!(
        response.trace.turns[1]
            .decisions
            .last()
            .expect("terminal turn")
            .action,
        SearchControllerAction::Terminate
    );
}

#[test]
fn controller_records_pruning_when_context_budget_is_exceeded() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("alpha.txt"),
        "alpha implementation details for bounded controller context",
    )
    .expect("write alpha corpus file");
    std::fs::write(
        corpus.path().join("beta.txt"),
        "beta implementation details for bounded controller context",
    )
    .expect("write beta corpus file");

    let plan = custom_lexical_plan("bounded-controller-lexical");
    let engine = Sift::builder().build();
    let response = engine
        .search_controller(
            SearchControllerRequest::new(
                plan,
                vec![
                    SearchTurnRequest::new(corpus.path(), "alpha")
                        .with_turn_id("turn-a")
                        .with_limit(1)
                        .with_shortlist(1),
                    SearchTurnRequest::new(corpus.path(), "beta")
                        .with_turn_id("turn-b")
                        .with_limit(1)
                        .with_shortlist(1),
                ],
            )
            .with_retained_artifact_limit(1),
        )
        .expect("bounded controller search");

    assert_eq!(response.state.retained_artifacts.len(), 1);
    assert!(
        response.state.retained_artifacts[0]
            .path
            .ends_with("beta.txt"),
        "fresh evidence should displace stale context under the budget"
    );
    assert!(
        response.trace.turns[0]
            .decisions
            .iter()
            .all(|decision| decision.action != SearchControllerAction::Prune)
    );
    assert!(
        response.trace.turns[1]
            .decisions
            .iter()
            .any(|decision| decision.action == SearchControllerAction::Prune)
    );
}

#[test]
fn embedded_facade_lowers_autonomous_planner_trace_into_controller_runtime() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("alpha.txt"),
        "alpha implementation details for the autonomous planner seam",
    )
    .expect("write alpha corpus file");

    let engine = Sift::builder().build();
    let response = engine
        .search_autonomous_with(
            AutonomousSearchRequest::new(corpus.path(), "find the alpha details")
                .with_strategy("bm25")
                .with_planner_strategy(AutonomousPlannerStrategy::heuristic())
                .with_limit(1)
                .with_shortlist(1),
            &SingleStepPlanner,
        )
        .expect("autonomous search with planner seam");

    assert_eq!(response.root_task, "find the alpha details");
    assert_eq!(response.turns.len(), 1);
    assert_eq!(response.turns[0].turn.turn_id, "turn-a");
    assert_eq!(response.turns[0].turn.strategy, "bm25");
    assert_eq!(response.trace.turns.len(), 1);
    assert!(response.state.completed);
    assert_eq!(response.planner_trace.steps.len(), 1);
    assert_eq!(
        response.planner_trace.steps[0].decisions[0].action,
        AutonomousPlannerAction::Search
    );
    assert_eq!(
        response.planner_trace.stop_reason,
        Some(AutonomousPlannerStopReason::GoalSatisfied)
    );
}

#[test]
fn embedded_facade_executes_built_in_heuristic_autonomous_runtime() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("alpha.txt"),
        "alpha runtime details for the built in autonomous facade",
    )
    .expect("write alpha corpus file");

    let engine = Sift::builder().build();
    let response = engine
        .search_autonomous(
            AutonomousSearchRequest::new(corpus.path(), "find alpha runtime details")
                .with_strategy("bm25")
                .with_limit(1)
                .with_shortlist(1),
        )
        .expect("built-in autonomous search");

    assert_eq!(response.turns.len(), 1);
    assert_eq!(response.turns[0].turn.turn_id, "turn-1");
    assert_eq!(response.turns[0].turn.strategy, "bm25");
    assert_eq!(
        response.planner_strategy,
        AutonomousPlannerStrategy::heuristic()
    );
    assert_eq!(
        response.planner_trace.planner_strategy,
        AutonomousPlannerStrategy::heuristic()
    );
    assert_eq!(
        response.planner_trace.steps[0].decisions[0].action,
        AutonomousPlannerAction::Search
    );
    assert_eq!(
        response.planner_trace.steps[0]
            .decisions
            .last()
            .expect("terminal planner decision")
            .action,
        AutonomousPlannerAction::Terminate
    );
    assert!(response.state.completed);
}

#[test]
fn built_in_autonomous_runtime_reuses_controller_retained_evidence_carryover() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("alpha.txt"),
        "alpha runtime details for the first autonomous turn",
    )
    .expect("write alpha corpus file");
    std::fs::write(
        corpus.path().join("beta.txt"),
        "beta evidence carryover details for the second autonomous turn",
    )
    .expect("write beta corpus file");

    let engine = Sift::builder().build();
    let response = engine
        .search_autonomous(
            AutonomousSearchRequest::new(corpus.path(), "alpha runtime")
                .with_strategy("bm25")
                .with_limit(1)
                .with_shortlist(1)
                .with_retained_artifact_limit(1)
                .with_state(
                    sift::AutonomousPlannerState::new(2).with_retained_artifacts(vec![
                        retained_artifact(
                            "seed-evidence",
                            "context/seed.txt",
                            "beta evidence carryover",
                            "carry beta into the next autonomous step",
                        ),
                    ]),
                ),
        )
        .expect("built-in autonomous search with retained evidence");

    assert_eq!(response.turns.len(), 2);
    assert_eq!(
        response.planner_trace.steps[1].decisions[0]
            .query
            .as_deref(),
        Some("beta evidence carryover context seed")
    );
    assert_eq!(response.trace.turns.len(), 2);
    assert_eq!(response.state.retained_artifacts.len(), 1);
    assert!(
        response.state.retained_artifacts[0]
            .path
            .ends_with("beta.txt"),
        "the shared controller budget should carry forward fresh evidence from the later turn"
    );
    assert!(
        response.trace.turns[1]
            .decisions
            .iter()
            .any(|decision| decision.action == SearchControllerAction::Prune),
        "the second turn should reuse shared controller retention semantics under the limit"
    );
}

#[test]
fn built_in_autonomous_runtime_advances_from_explicit_planner_state() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("alpha.txt"),
        "alpha runtime details for the resumed autonomous turn",
    )
    .expect("write alpha corpus file");
    std::fs::write(
        corpus.path().join("beta.txt"),
        "beta evidence carryover details for the resumed autonomous follow-up",
    )
    .expect("write beta corpus file");

    let engine = Sift::builder().build();
    let response = engine
        .search_autonomous(
            AutonomousSearchRequest::new(corpus.path(), "alpha runtime")
                .with_strategy("bm25")
                .with_limit(1)
                .with_shortlist(1)
                .with_state(
                    sift::AutonomousPlannerState::new(3)
                        .with_current_step(
                            sift::AutonomousPlannerStepCursor::new("step-2", 2)
                                .with_parent_step_id("step-1"),
                        )
                        .with_retained_artifacts(vec![retained_artifact(
                            "resume-evidence",
                            "context/seed.txt",
                            "beta evidence carryover",
                            "advance from explicit planner state",
                        )]),
                ),
        )
        .expect("built-in autonomous search resumed from explicit planner state");

    assert_eq!(response.turns.len(), 1);
    assert_eq!(response.turns[0].turn.turn_id, "turn-2");
    assert_eq!(response.planner_trace.steps[0].step.step_id, "step-2");
    assert_eq!(response.trace.turns.len(), 1);
    assert_eq!(
        response.trace.turns[0].decisions[0].action,
        SearchControllerAction::Retrieve
    );
    assert_eq!(response.state.current_step.step_id, "step-2");
    assert!(response.state.completed);
}

#[test]
fn built_in_autonomous_runtime_resumes_from_explicit_planner_state() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("beta.txt"),
        "retry loop adapter layer context seed beta runtime details",
    )
    .expect("write beta corpus file");

    let engine = Sift::builder().build();
    let response = engine
        .search_autonomous(
            AutonomousSearchRequest::new(corpus.path(), "find alpha details")
                .with_strategy("bm25")
                .with_limit(1)
                .with_shortlist(1)
                .with_state(
                    sift::AutonomousPlannerState::new(3)
                        .with_current_step(
                            sift::AutonomousPlannerStepCursor::new("step-2", 2)
                                .with_parent_step_id("step-1"),
                        )
                        .with_retained_artifacts(vec![retained_artifact(
                            "seed-evidence",
                            "context/seed.txt",
                            "retry loop adapter layer",
                            "resume from retained evidence",
                        )]),
                ),
        )
        .expect("resumed built-in autonomous search");

    assert_eq!(response.turns.len(), 1);
    assert_eq!(response.turns[0].turn.turn_id, "turn-2");
    assert_eq!(response.planner_trace.steps.len(), 1);
    assert_eq!(response.planner_trace.steps[0].step.step_id, "step-2");
    assert_eq!(response.planner_trace.steps[0].step.sequence, 2);
    assert_eq!(
        response.planner_trace.steps[0].decisions[0]
            .query
            .as_deref(),
        Some("retry loop adapter layer context seed")
    );
    assert_eq!(
        response.planner_trace.stop_reason,
        Some(AutonomousPlannerStopReason::NoAdditionalEvidence)
    );
}

#[test]
fn built_in_autonomous_runtime_routes_model_driven_strategy_selection() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("alpha.txt"),
        "alpha runtime details for the model driven autonomous turn",
    )
    .expect("write alpha corpus file");

    let model_output = r#"{
        "steps": [
            {
                "step": {
                    "step_id": "step-1",
                    "parent_step_id": null,
                    "sequence": 1
                },
                "decisions": [
                    {
                        "action": "search",
                        "rationale": "model-driven planner selected the most salient token",
                        "query": "alpha runtime details",
                        "turn_id": "turn-md-1",
                        "next_step": null,
                        "stop_reason": null
                    },
                    {
                        "action": "terminate",
                        "rationale": "the root task is satisfied after the first search",
                        "query": null,
                        "turn_id": null,
                        "next_step": null,
                        "stop_reason": "goal-satisfied"
                    }
                ]
            }
        ],
        "completed": true,
        "stop_reason": "goal-satisfied"
    }"#;

    let engine = Sift::builder()
        .with_generative_model(std::sync::Arc::new(StaticGenerativeModel {
            output: model_output.to_string(),
        }))
        .build();
    let response = engine
        .search_autonomous(
            AutonomousSearchRequest::new(corpus.path(), "find alpha details")
                .with_strategy("bm25")
                .with_planner_strategy(
                    AutonomousPlannerStrategy::model_driven().with_profile("local-planner-v1"),
                )
                .with_limit(1)
                .with_shortlist(1),
        )
        .expect("model-driven autonomous search");

    assert_eq!(response.turns.len(), 1);
    assert_eq!(response.turns[0].turn.turn_id, "turn-md-1");
    assert_eq!(
        response.planner_strategy,
        AutonomousPlannerStrategy::model_driven().with_profile("local-planner-v1")
    );
    assert_eq!(
        response.planner_trace.stop_reason,
        Some(AutonomousPlannerStopReason::GoalSatisfied)
    );
    assert_eq!(
        response.planner_trace.steps[0].decisions[0].action,
        AutonomousPlannerAction::Search
    );
}

#[test]
fn built_in_autonomous_runtime_reports_unavailable_model_driven_profiles() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(corpus.path().join("alpha.txt"), "alpha runtime details")
        .expect("write alpha corpus file");

    let engine = Sift::builder().build();
    let error = engine
        .search_autonomous(
            AutonomousSearchRequest::new(corpus.path(), "find alpha details")
                .with_strategy("bm25")
                .with_planner_strategy(
                    AutonomousPlannerStrategy::model_driven().with_profile("missing-profile"),
                )
                .with_limit(1)
                .with_shortlist(1),
        )
        .expect_err("missing model-driven profile should fail explicitly");

    assert!(
        error
            .to_string()
            .contains("failed to resolve model-driven planner profile")
    );
}

#[test]
fn built_in_autonomous_runtime_executes_bounded_graph_mode() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("alpha.txt"),
        "alpha runtime details for the graph root branch",
    )
    .expect("write alpha corpus file");
    std::fs::write(
        corpus.path().join("beta.txt"),
        "beta evidence carryover details for the graph follow up branch",
    )
    .expect("write beta corpus file");

    let engine = Sift::builder().build();
    let response = engine
        .search_autonomous(
            AutonomousSearchRequest::new(corpus.path(), "alpha runtime")
                .with_mode(AutonomousSearchMode::Graph)
                .with_strategy("bm25")
                .with_limit(1)
                .with_shortlist(1)
                .with_retained_artifact_limit(1)
                .with_state(sift::AutonomousPlannerState::new(2).with_retained_artifacts(vec![
                    retained_artifact(
                        "seed-evidence",
                        "context/seed.txt",
                        "beta evidence carryover",
                        "fork a follow-up graph branch",
                    ),
                ])),
        )
        .expect("built-in graph autonomous search");

    assert_eq!(response.mode, AutonomousSearchMode::Graph);
    assert_eq!(response.turns.len(), 2);
    assert_eq!(response.turns[0].turn.turn_id, "turn-1");
    assert_eq!(response.turns[1].turn.turn_id, "turn-2");
    assert!(response.state.completed);

    let graph_episode = response
        .state
        .graph_episode
        .as_ref()
        .expect("graph episode state");
    assert_eq!(graph_episode.root_node_id.as_deref(), Some("step-1"));
    assert_eq!(graph_episode.branches.len(), 3);
    assert!(graph_episode.frontier.is_empty());
    assert_eq!(graph_episode.active_branch_id.as_deref(), Some("branch-3"));
    assert!(
        graph_episode
            .branches
            .iter()
            .find(|branch| branch.branch_id == "branch-3")
            .expect("graph follow-up branch")
            .retained_artifacts[0]
            .path
            .ends_with("beta.txt")
    );
    assert!(
        response.trace.turns[1]
            .decisions
            .iter()
            .any(|decision| decision.action == SearchControllerAction::Prune),
        "branch-local retained evidence should still honor the bounded controller budget"
    );
}

#[test]
fn graph_autonomous_runtime_surfaces_explicit_trace_contract_errors() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(corpus.path().join("alpha.txt"), "alpha runtime details")
        .expect("write alpha corpus file");

    let engine = Sift::builder().build();
    let error = engine
        .search_autonomous_with(
            AutonomousSearchRequest::new(corpus.path(), "find alpha details")
                .with_mode(AutonomousSearchMode::Graph)
                .with_strategy("bm25")
                .with_limit(1)
                .with_shortlist(1),
            &InvalidGraphPlanner,
        )
        .expect_err("invalid graph trace should fail explicitly");

    assert!(
        error
            .to_string()
            .contains("graph trace contract error"),
        "unexpected graph runtime error: {error}"
    );
}

#[test]
fn built_in_graph_autonomous_runtime_routes_model_driven_strategy_selection() {
    let corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        corpus.path().join("alpha.txt"),
        "alpha runtime details for the model driven graph branch",
    )
    .expect("write alpha corpus file");

    let model_output = r#"{
        "steps": [
            {
                "step": {
                    "step_id": "step-1",
                    "parent_step_id": null,
                    "sequence": 1
                },
                "decisions": [
                    {
                        "action": "fork",
                        "rationale": "model-driven planner forked the strongest graph branch",
                        "query": "alpha runtime details",
                        "turn_id": null,
                        "branch_id": "branch-root",
                        "node_id": "step-1",
                        "target_branch_id": "branch-a",
                        "target_node_id": "node-a",
                        "edge_id": "edge-a",
                        "edge_kind": "child",
                        "frontier_id": "frontier-a",
                        "next_step": {
                            "step_id": "node-a",
                            "parent_step_id": "step-1",
                            "sequence": 2
                        },
                        "stop_reason": null
                    },
                    {
                        "action": "select",
                        "rationale": "choose the forked graph branch",
                        "query": null,
                        "turn_id": null,
                        "branch_id": "branch-a",
                        "node_id": "node-a",
                        "target_branch_id": null,
                        "target_node_id": null,
                        "edge_id": null,
                        "edge_kind": null,
                        "frontier_id": "frontier-a",
                        "next_step": null,
                        "stop_reason": null
                    },
                    {
                        "action": "search",
                        "rationale": "execute the forked branch query",
                        "query": "alpha runtime details",
                        "turn_id": "turn-md-graph-1",
                        "branch_id": "branch-a",
                        "node_id": "node-a",
                        "target_branch_id": null,
                        "target_node_id": null,
                        "edge_id": null,
                        "edge_kind": null,
                        "frontier_id": null,
                        "next_step": null,
                        "stop_reason": null
                    },
                    {
                        "action": "terminate",
                        "rationale": "the graph branch satisfied the root task",
                        "query": null,
                        "turn_id": null,
                        "branch_id": "branch-a",
                        "node_id": null,
                        "target_branch_id": null,
                        "target_node_id": null,
                        "edge_id": null,
                        "edge_kind": null,
                        "frontier_id": null,
                        "next_step": null,
                        "stop_reason": "goal-satisfied"
                    }
                ]
            }
        ],
        "completed": true,
        "stop_reason": "goal-satisfied"
    }"#;

    let engine = Sift::builder()
        .with_generative_model(std::sync::Arc::new(StaticGenerativeModel {
            output: model_output.to_string(),
        }))
        .build();
    let response = engine
        .search_autonomous(
            AutonomousSearchRequest::new(corpus.path(), "find alpha details")
                .with_mode(AutonomousSearchMode::Graph)
                .with_strategy("bm25")
                .with_planner_strategy(
                    AutonomousPlannerStrategy::model_driven().with_profile("local-planner-v1"),
                )
                .with_limit(1)
                .with_shortlist(1),
        )
        .expect("model-driven graph autonomous search");

    assert_eq!(response.mode, AutonomousSearchMode::Graph);
    assert_eq!(response.turns.len(), 1);
    assert_eq!(response.turns[0].turn.turn_id, "turn-md-graph-1");
    assert_eq!(
        response.planner_strategy,
        AutonomousPlannerStrategy::model_driven().with_profile("local-planner-v1")
    );
    assert_eq!(
        response.planner_trace.steps[0].decisions[0].action,
        AutonomousPlannerAction::Fork
    );
    assert_eq!(
        response.planner_trace.stop_reason,
        Some(AutonomousPlannerStopReason::GoalSatisfied)
    );
}
