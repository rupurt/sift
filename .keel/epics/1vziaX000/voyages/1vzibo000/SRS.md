# Instrumentation Foundation - Software Requirements Specification

> Implement the Telemetry module, tracing integration, and benchmarking recipes.

## Scope

### In Scope

- [SCOPE-01] Create a `Telemetry` struct to track cache hits and misses.
- [SCOPE-02] Replace custom `trace!` macro with `tracing` crate.
- [SCOPE-03] Configure `tracing-subscriber` for CLI control.
- [SCOPE-04] Add `just` tasks for flamegraphs and micro-benchmarks.

### Out of Scope

- [SCOPE-05] Remote telemetry collection.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-09 | A global/shared `Telemetry` object must record cache state transitions (hit/miss). | SCOPE-01 | FR-01 | Trace verification in `-vv` output. |
| SRS-10 | Benchmark reports must display cache hit percentage at `-v` level. | SCOPE-01 | FR-01 | Empirical CLI check. |
| SRS-11 | Search phases (extraction, embedding, retrieval) must be instrumented with `tracing::span`. | SCOPE-02 | FR-02 | Code inspection. |
| SRS-12 | A `benches/` directory must contain a `criterion` harness for `tokenize`. | SCOPE-04 | FR-03 | Run `cargo bench`. |
| SRS-13 | Provide a `just` recipe for generating SVG flamegraphs. | SCOPE-04 | FR-03 | Run `just bench-flamegraph`. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-14 | Telemetry recording must be thread-safe (Atomic counters). | SCOPE-01 | NFR-01 | Code inspection. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
