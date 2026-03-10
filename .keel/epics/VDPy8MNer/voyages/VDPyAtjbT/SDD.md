# Query Embedding and Memory Allocation Optimization - SDD

> Implement architectural refinements to caching and scoring to improve search throughput.

## Architecture Overview

The optimization strategy focuses on three layers:
1. **Application Layer:** Introducing a session-scoped `QueryEmbeddingCache` to eliminate redundant inference for repeated queries.
2. **Domain Layer:** Refactoring the retrieval pipeline to use pre-allocated buffers and avoid heap allocations in critical scoring paths.
3. **Hardware Adaptation Layer:** Optimizing the inner loop of vector similarity calculations (`dot_product`) using architecture-specific SIMD instructions.

## Components

### `QueryEmbeddingCache`
A session-level cache implemented as an in-memory store (e.g., `HashMap<String, Vec<f32>>`) to store query embeddings. It is owned by the search orchestration layer and passed to retrievers.

### `DenseReranker` (Adapter)
Updated to intercept `embed` calls and check the `QueryEmbeddingCache` before delegating to the Candle inference engine.

### `VectorRetriever` (Domain)
Refactored to use pre-allocated buffers for `SegmentHit` results, avoiding per-segment or per-document allocations during the inner scoring loop.

## Data Flow

1. **Search Initiation:** A `SearchRequest` is received, carrying a `QueryEmbeddingCache`.
2. **Retrieval Phase:** 
   - `VectorRetriever` requests query embedding.
   - `DenseReranker` checks `QueryEmbeddingCache`.
   - On miss: Candle runs inference, results are cached.
   - On hit: Cached embedding is returned instantly.
3. **Scoring Loop:**
   - `VectorRetriever` pre-allocates a results vector.
   - `dot_product` is executed using SIMD instructions on available hardware.
   - `SegmentHit` objects are collected into the pre-allocated buffer.
4. **Aggregation & Fusion:** Candidate lists are produced and fused using existing logic, with minimized allocations.

## Design Approach

### Query Embedding Caching

We will introduce a `QueryEmbeddingCache` at the application level to store already-computed embeddings for search queries. 
- **Storage:** A `HashMap<String, Vec<f32>>` protected by a `RwLock` or similar if needed for multi-threaded access.
- **Integration:** The `SearchRequest` or the `SearchService` will hold the cache across searches. For the first slice, we'll implement it as a session-level cache passed through the search execution.

### Memory Allocation Optimization

We will refactor `score_segments_manually` and its callers to pre-allocate vectors for hits and candidates based on the corpus size.
- **Pre-allocation:** `Vec::with_capacity(segments.len())` for `SegmentHit` in `score_segments_manually`.
- **Reusable Buffers:** If possible, pass reusable buffers through the retrieval phase to avoid re-allocating for each retriever.

### SIMD-Optimized Dot Product

We will provide an optimized `dot_product` function.
- **Crate:** We'll use the `wide` crate or equivalent for SIMD abstractions to ensure portability and efficiency on supported architectures.
- **Fallbacks:** Provide a scalar implementation for unsupported hardware.

## Deployment Strategy

- Incrementally apply changes to `vector.rs` and `dense.rs`.
- Verify performance after each change using `sift search -vv`.
- Run micro-benchmarks to quantify the SIMD speedup.
