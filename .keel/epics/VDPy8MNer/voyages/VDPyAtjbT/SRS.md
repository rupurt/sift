# Query Embedding and Memory Allocation Optimization - SRS

> Optimize search performance by implementing a query embedding cache and reducing memory allocations in the core retrieval pipeline.

## Scope

### In Scope

- [SCOPE-01] Implement a `QueryEmbeddingCache` at the session level.
- [SCOPE-02] Modify `DenseReranker` to check the cache before re-embedding queries.
- [SCOPE-03] Pre-allocate `CandidateList` and `SegmentHit` vectors in the retrieval inner loop.
- [SCOPE-04] Implement SIMD-optimized `dot_product` using `wide` or equivalent crate.

### Out of Scope

- [SCOPE-05] Moving query caching to a persistent disk-based store (out of scope for first slice).
- [SCOPE-06] Rewriting the entire segment aggregation logic (only optimize allocations).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Verification |
|----|-------------|--------|-------|--------------|
| SRS-01 | `DenseReranker` must query a session-level `QueryEmbeddingCache` before running Candle inference. | FR-01 | SCOPE-01, SCOPE-02 | board: VDPyDiXga |
| SRS-02 | `score_segments_manually` must use pre-allocated vectors for `SegmentHit` and other intermediate results. | FR-03 | SCOPE-03 | manual: Allocation profiling with `dhat` or equivalent |
| SRS-03 | `dot_product` must leverage SIMD instructions on x86_64 and aarch64 architectures. | FR-02 | SCOPE-04 | board: VDPyDiqht |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Verification |
|----|-------------|--------|-------|--------------|
| SRS-04 | Performance gains for query caching must be observable in `sift search -vv` output via tracing spans. | NFR-01 | SCOPE-01, SCOPE-02 | manual: CLI trace output inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Verify query cache hits with `sift search -vv` on repeated searches within the same session or via a mock integration test.
- Use `cargo bench` to compare the performance of `dot_product` with and without SIMD.
- Use `dhat` to count allocations in `score_segments_manually`.
