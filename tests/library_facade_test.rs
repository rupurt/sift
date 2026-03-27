use sift::{
    Fusion, FusionPolicy, QueryExpansionPolicy, Reranking, RerankingPolicy, Retriever,
    RetrieverPolicy, SearchControllerAction, SearchControllerRequest, SearchEmission,
    SearchEmissionMode, SearchInput, SearchOptions, SearchPlan, SearchTurnRequest, Sift,
};

fn custom_lexical_plan(name: &str) -> SearchPlan {
    SearchPlan {
        name: name.to_string(),
        query_expansion: QueryExpansionPolicy::None,
        retrievers: vec![RetrieverPolicy::Bm25],
        fusion: FusionPolicy::Rrf,
        reranking: RerankingPolicy::None,
    }
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

    assert_eq!(response.results.len(), 1);
    assert!(response.results[0].path.ends_with("guide.txt"));
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
            assert!(view.results[0].path.ends_with("guide.txt"));
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
            .with_retained_evidence_limit(2),
        )
        .expect("controller search with explicit plan");

    assert_eq!(response.plan.name, "controller-lexical");
    assert_eq!(response.turns.len(), 2);
    assert_eq!(response.trace.turns.len(), 2);
    assert!(response.state.completed);
    assert_eq!(response.state.next_turn, 2);
    assert!(response.state.retained_evidence.len() <= 2);
    assert_eq!(response.turns[0].turn.strategy, "controller-lexical");
    assert_eq!(response.turns[1].turn.strategy, "controller-lexical");
    assert!(!response.turns[1].turn.retained_evidence.is_empty());
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
            .with_retained_evidence_limit(1),
        )
        .expect("bounded controller search");

    assert_eq!(response.state.retained_evidence.len(), 1);
    assert!(
        response.state.retained_evidence[0]
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
