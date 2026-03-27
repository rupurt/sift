use sift::{
    Fusion, Reranking, Retriever, SearchControllerAction, SearchEmission, SearchEmissionMode,
    SearchInput, SearchOptions, SearchTurnRequest, Sift,
};

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
