# Comprehensive Performance Instrumentation — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | Vital for transforming optimization from guesswork into data-driven engineering. |
| Confidence | 5 | All recommended tools (tracing, dhat, criterion) are stable and industry-standard. |
| Effort | 3 | Requires systemic changes to logging and adding new benchmark crates. |
| Risk | 1 | Low risk; instrumentation can be disabled or stripped in release builds. |

*Scores range from 1-5 (1=Very Low, 5=Very High)*

## Analysis

### Findings
- Structured tracing is required to visualize the "waterfall" of search phases [SRC-01].
- Allocation profiling will identify memory bottlenecks in the text extraction pipeline [SRC-02].
- Micro-benchmarks are necessary to protect the performance of hot-path functions like BM25 [SRC-03].

### Dependencies
- Access to `cargo-flamegraph` on the developer machine for the new just task [SRC-01].
- Integration of `tracing` and `tracing-subscriber` crates [SRC-01].

### Alternatives Considered
- **Keep custom `trace!` macro:** Rejected because it doesn't provide structured spans or compatible output for external visualization tools [SRC-01].
- **Strictly wall-clock timing:** Rejected because it doesn't reveal *why* a phase is slow (e.g., waiting on I/O vs. compute) [SRC-01] [SRC-03].

## Recommendation

- [x] Proceed → convert to epic [SRC-01] [SRC-02] [SRC-03]
- [ ] Park → revisit later
- [ ] Decline → document learnings

Proceed with an Epic to implement "Performance Observability" by integrating `tracing`, `dhat`, and `criterion`.
