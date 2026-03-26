use sift::internal::{
    search::{CachedEmbedder, Embedder},
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
    let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let embedder = CachedEmbedder {
        inner: Arc::new(MockEmbedder {
            call_count: call_count.clone(),
        }),
        cache: Arc::new(RwLock::new(HashMap::new())),
    };
    let query = "performance optimization".to_string();

    let first = embedder
        .embed(std::slice::from_ref(&query))
        .expect("first embed");
    let count1 = call_count.load(std::sync::atomic::Ordering::SeqCst);

    let second = embedder
        .embed(std::slice::from_ref(&query))
        .expect("second embed");
    let count2 = call_count.load(std::sync::atomic::Ordering::SeqCst);

    assert_eq!(first, second);
    assert_eq!(count1, 1, "first embed should reach the inner embedder once");
    assert_eq!(
        count1, count2,
        "Call count should not increase on second embed due to query caching"
    );
}

#[test]
fn test_query_embedding_cache_handles_mixed_cache_hits() {
    let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let cache = Arc::new(RwLock::new(HashMap::new()));
    let embedder = CachedEmbedder {
        inner: Arc::new(MockEmbedder {
            call_count: call_count.clone(),
        }),
        cache: cache.clone(),
    };

    cache
        .write()
        .expect("cache write")
        .insert("cached".to_string(), vec![0.2; 384]);

    let embeddings = embedder
        .embed(&["cached".to_string(), "fresh".to_string()])
        .expect("mixed embed");

    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(embeddings[0], vec![0.2; 384]);
    assert_eq!(embeddings[1], vec![0.1; 384]);
    assert_eq!(
        cache.read().expect("cache read").get("fresh"),
        Some(&vec![0.1; 384])
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
