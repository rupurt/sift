use sift::internal::{
    dense::DenseModelSpec,
    search::{Embedder, LocalFileCorpusRepository, SearchRequest, run_search},
    system::Telemetry,
    vector::dot_product,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

pub struct MockEmbedder {
    pub call_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl Embedder for MockEmbedder {
    fn embed(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        self.call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(texts.iter().map(|_| vec![0.1; 384]).collect())
    }
    fn dimension(&self) -> usize {
        384
    }
}

#[test]
fn test_query_embedding_cache_avoids_redundant_calls() {
    let temp_corpus = tempfile::tempdir().expect("temp corpus");
    std::fs::write(
        temp_corpus.path().join("test.txt"),
        "performance optimization",
    )
    .expect("write test file");

    let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let embedder = Arc::new(MockEmbedder {
        call_count: call_count.clone(),
    });
    let query_cache = Arc::new(RwLock::new(HashMap::new()));
    let telemetry = Arc::new(Telemetry::new());

    let request = SearchRequest {
        strategy: "vector".to_string(),
        query: "performance".to_string(),
        path: temp_corpus.path().to_path_buf(),
        limit: 10,
        shortlist: 10,
        dense_model: DenseModelSpec::default(),
        rerank_model: None,
        verbose: 0,
        retrievers: None,
        fusion: None,
        reranking: None,
        telemetry: telemetry.clone(),
        cache_dir: None,
        query_cache: Some(query_cache.clone()),
    };

    // First search: should call embedder for query
    let _response1 = run_search(
        &request,
        None,
        &LocalFileCorpusRepository,
        Some(embedder.clone()),
    )
    .expect("first search");
    let count1 = call_count.load(std::sync::atomic::Ordering::SeqCst);
    // Note: It might call embed twice if it also embeds the document segment.
    // In our case, the document segment is not in the cache, so it will be embedded.
    // Query embedding is the first call, segment embedding is the second call.
    assert!(count1 >= 1);

    // Second search with same query: should NOT call embedder for query
    let _response2 = run_search(
        &request,
        None,
        &LocalFileCorpusRepository,
        Some(embedder.clone()),
    )
    .expect("second search");
    let count2 = call_count.load(std::sync::atomic::Ordering::SeqCst);

    // The query embedding should be cached. The document embedding will also be cached because
    // run_search uses the heuristic cache (mtime didn't change).
    // So the call count should NOT increase.
    assert_eq!(
        count1, count2,
        "Call count should not increase on second search due to query and segment caching"
    );
}

#[test]
fn test_dot_product_consistency() {
    let a = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    let b = vec![1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1];

    let expected: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let actual = dot_product(&a, &b);

    assert!(
        (actual - expected).abs() < 1e-6,
        "SIMD dot_product should be consistent with scalar version. Got {}, expected {}",
        actual,
        expected
    );
}
