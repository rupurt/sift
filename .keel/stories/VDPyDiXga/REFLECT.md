# Reflection - Implement Session-Level Query Embedding Cache

We implemented a decorator-based `CachedEmbedder` that wraps any `Embedder` and uses a session-level `QueryEmbeddingCache` (Arc<RwLock<HashMap<String, Vec<f32>>>>). This correctly identifies and reuses embeddings for identical queries, significantly reducing redundant inference costs in multi-search scenarios (like evaluations or agentic loops). The design is clean and maintains hexagonal boundaries.
