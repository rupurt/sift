# Performance Observability - Product Requirements

> Transform performance optimization from "guesswork based on wall-clock time" into a data-driven process by integrating structured telemetry, allocation profiling, and cache efficiency metrics.

## Problem Statement

As `sift` scales to larger repositories (thousands of files), the complex interactions between filesystem I/O, BLAKE3 hashing, document extraction, and neural network inference become difficult to debug. We have experienced "stuck" states and high latency without a clear view of where the time is spent or where thread contention occurs.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Quantify cache effectiveness | Displayed hit rates | % of work skipped is clearly visible in benchmarks. |
| GOAL-02 | Visualize pipeline bottlenecks | Tracing spans | Major search phases are instrumented with structured spans. |
| GOAL-03 | Detect memory/compute regressions | Micro-benchmarks | Standard CI-ready benchmarks for hot-path functions. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | Improving sift performance | Precise data on where latency/allocations occur. |

## Scope

### In Scope

- [SCOPE-01] Integrate `tracing` and `tracing-subscriber` for structured spans.
- [SCOPE-02] Implement `Telemetry` module for cache metrics.
- [SCOPE-03] Establish `criterion` micro-benchmarks.
- [SCOPE-04] Add `just` recipes for flamegraphs and heap profiling.

### Out of Scope

- [SCOPE-05] Remote telemetry collection.
- [SCOPE-06] External dashboard integration.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | `sift` must display Heuristic, Blob, and Embedding hit rates in benchmark reports. | GOAL-01 | must | Validates cache effectiveness. |
| FR-02 | The search pipeline must be wrapped in `tracing` spans. | GOAL-02 | must | Enables waterfall visualization. |
| FR-03 | Hot-path functions (BM25, dot_product) must have `criterion` benchmarks. | GOAL-03 | should | Prevents algorithmic regressions. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Instrumentation overhead must be < 5% of total search time. | GOAL-02 | must | Monitoring should not become the bottleneck. |
| NFR-02 | Tracing must support a "silent" mode for standard users. | GOAL-02 | must | Maintains clean CLI output. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Verify telemetry correctness by running `sift search -vv` on known files and checking hit rates.
- Benchmark the overhead of `tracing` spans using `criterion`.
- Generate and inspect SVG flamegraphs for SciFact benchmarks.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| `tracing` overhead is negligible | Search becomes slower | Micro-benchmarks |
| `AtomicUsize` is sufficient for telemetry | Potential overflow | Analysis of total document counts |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Will tracing output clutter CLI? | Engineering | Mitigation: Map verbose level to tracing level |
| Criterion build time | Engineering | Track impact on dev cycle |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Telemetry module implemented and integrated into benchmark output.
- [ ] `tracing` spans visible at `-vv` level.
- [ ] `criterion` benchmarks passing in `target/criterion`.
- [ ] `just bench flamegraph` generates a valid SVG.
<!-- END SUCCESS_CRITERIA -->
