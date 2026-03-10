# High-Performance Retrieval Refinements - Product Requirements

> This epic focuses on optimizing the core search pipeline of `sift` to ensure sub-second latency even as the local document corpus grows to thousands of files. We will achieve this through query embedding caching, SIMD-optimized scoring, and architectural refinements for memory and I/O efficiency.

## Problem Statement

Current vector retrieval is bottlenecked by query embedding latency (approx. 20-30ms per search) and unoptimized dot-product calculations. While heuristic caching correctly skips document extraction and embedding, the runtime scoring phase does not yet leverage the full potential of modern CPUs (SIMD, pre-allocation) or OS-level optimizations (mmap for large blob access).

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Eliminate redundant query embedding | Query embedding cache hit rate | 0ms embedding time for repeated identical queries in a session. |
| GOAL-02 | Optimize dot-product performance | Micro-benchmark throughput | 2x-5x speedup in raw dot-product calculations using SIMD/BLAS. |
| GOAL-03 | Reduce runtime allocations in scoring | Allocation profiling | Near-zero allocations in the inner loop of `score_segments`. |
| GOAL-04 | Efficient access to pre-computed embeddings | I/O latency | Loading 10,000+ embeddings from cache is sub-10ms using mmap. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | Searches large local repositories | Near-instant search results that don't disrupt flow. |
| Agent | Performs repeated searches in a loop | High-throughput retrieval without compounding latency. |

## Scope

### In Scope

- [SCOPE-01] Implement a session-level or persistent query embedding cache.
- [SCOPE-02] Introduce SIMD-optimized dot-product implementations (e.g., using `wide` or `packed_simd` or `ndarray`).
- [SCOPE-03] Pre-allocate buffers for scoring and fusion to avoid runtime allocations.
- [SCOPE-04] Explore `mmap` for reading large embedding blobs from the global cache.

### Out of Scope

- [SCOPE-05] Implementing a background indexing service.
- [SCOPE-06] Moving inference to the GPU (staying "Single Binary/CPU-First" for now).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Sift must cache query embeddings to avoid re-running Candle inference for identical queries. | GOAL-01 | must | Query embedding is a fixed cost that can be eliminated for repeated searches. |
| FR-02 | The vector retriever must use SIMD-accelerated dot-product calculations where supported by the hardware. | GOAL-02 | must | Essential for scaling to large corpora with many segments. |
| FR-03 | Search execution must use pre-allocated pools/buffers for intermediate candidate lists and scores. | GOAL-03 | should | Reduces pressure on the allocator during search. |
| FR-04 | The global blob store must support fast random access to embeddings, potentially via mmap, to minimize loading I/O. | GOAL-04 | should | Large corpora require efficient loading of many small embeddings. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Performance improvements must be documented with "Before" and "After" evidence in story acceptance criteria. | GOAL-02 | must | Ensures optimizations are actually effective. |
| NFR-02 | The binary size should not increase significantly (>10%) due to new performance libraries. | GOAL-03 | should | Maintains sift's lightweight nature. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Use `cargo bench` and `criterion` to measure throughput of `dot_product`.
- Use `sift search -vv` to verify query embedding cache hits.
- Use `valgrind --tool=massif` or `dhat` to verify allocation reductions.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Query embedding latency is primarily due to inference overhead rather than tokenization. | Caching embeddings might not be the most effective optimization. | Profile tokenization vs inference. |
| Modern CPU architectures (x86_64, aarch64) provide SIMD instructions that are accessible via Rust libraries. | SIMD speedup might not be portable or available. | Verify library support for target architectures. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the query cache be persistent across processes or only session-level? | Product | Session-level for the first slice. |
| Does `mmap` provide a significant win over standard file I/O for the current blob size? | Engineering | Needs benchmarking for large corpora. |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Query embedding cache implemented and verified with `-vv`.
- [ ] SIMD-optimized dot-product shows significant speedup in micro-benchmarks.
- [ ] Large corpus search (1000+ files) remains sub-second on standard hardware.
<!-- END SUCCESS_CRITERIA -->
