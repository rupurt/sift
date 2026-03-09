# Comprehensive Performance Instrumentation - Product Requirements

> By integrating structured telemetry (spans and events), allocation profiling, and cache efficiency metrics, we can transform performance optimization from "guesswork based on wall-clock time" into a data-driven process. This will allow us to identify precise bottlenecks in the search asset pipeline and validate the impact of future optimizations (like parallelization or I/O changes).

## Problem Statement

As `sift` scales to larger repositories (thousands of files), the complex interactions between filesystem I/O, BLAKE3 hashing, document extraction, and neural network inference become difficult to debug. We have experienced "stuck" states and high latency without a clear view of where the time is spent or where thread contention occurs.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Validate bearing recommendation in delivery flow | Adoption signal | Initial rollout complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Product/Delivery Owner | Coordinates planning and execution | Reliable strategic direction |

## Scope

### In Scope

- Deliver the bearing-backed capability slice for this epic.

### Out of Scope

- Unrelated platform-wide refactors outside bearing findings.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Implement the core user workflow identified in bearing research. | GOAL-01 | must | Converts research recommendation into executable product capability. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Ensure deterministic behavior and operational visibility for the delivered workflow. | GOAL-01 | must | Keeps delivery safe and auditable during rollout. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove functional behavior through story-level verification evidence mapped to voyage requirements.
- Validate non-functional posture with operational checks and documented artifacts.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Bearing findings reflect current user needs | Scope may need re-planning | Re-check feedback during first voyage |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which rollout constraints should gate broader adoption? | Product | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Implement a `Telemetry` module to track and display cache hit rates (heuristic, blob, embedding).
- [ ] Integrate the `tracing` crate to provide structured spans for all major search phases.
- [ ] Establish a micro-benchmarking harness using `criterion` for hot-path functions.
- [ ] Integrate allocation profiling (e.g., `dhat`) into the development workflow.
- [ ] Provide a `just` recipe for generating flamegraphs to identify compute bottlenecks.
- [ ] Ensure all instrumentation is lightweight and does not significantly degrade baseline performance.
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

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

---

*This PRD was seeded from bearing `1vziaX000`. See `bearings/1vziaX000/` for original research.*
