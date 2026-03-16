# Comprehensive Performance Instrumentation — Evidence & Survey

## Overview

We will integrate standard Rust telemetry and benchmarking tools (`tracing`, `dhat`, `criterion`) to provide a data-driven path for performance optimization.

## Feasibility

The proposed tools are all mature, pure-Rust crates that integrate directly into the existing `cargo` toolchain. `tracing` is the defacto standard for structured logging [SRC-01], `dhat` is the canonical heap profiler [SRC-02], and `criterion` is the most widely used micro-benchmarking library [SRC-03]. All are feasible.

## Key Findings

- Structured tracing with `tracing` will allow us to visualize the "waterfall" latency of the search pipeline [SRC-01].
- Heap profiling with `dhat` is the most effective method for identifying memory "hot spots" in text-heavy processing [SRC-02].
- Micro-benchmarks with `criterion` are essential for preventing performance regressions in critical, low-level functions like BM25 scoring [SRC-03].

## Unknowns

- What is the overhead of including `tracing` spans in a release build?
- Can `dhat` be run effectively in CI to detect allocation regressions automatically?

## Instrumentation Strategies for Sift

### 1. Structured Spans (`tracing` crate)
The `tracing` crate is the industry standard for Rust telemetry. It provides:
- **Spans:** Represent a period of time (e.g., "Corpus Loading").
- **Events:** Represent a point in time (e.g., "Cache Hit").
- **Subscribers:** Can output to stderr, log files, or OpenTelemetry-compatible sinks.

**Implementation Plan:** Replace the custom `trace!` macro with `tracing` macros. Add a `tracing-subscriber` to `main.rs`.

### 2. Cache Efficiency Metrics (`Telemetry` Module)
We need to capture the effectiveness of our Zig-style cache.
- **Heuristic Hit Rate:** `hits / total_files_walked`
- **Blob Hit Rate:** `blob_hits / hash_checks`
- **Inference Bypass Rate:** `cached_embeddings / total_segments`

**Implementation Plan:** Create a thread-local or shared `Telemetry` struct that accumulates these counters during `load_search_corpus`.

### 3. Allocation Profiling (`dhat`)
`dhat` provides precise data on heap allocations.
- **Benefit:** Identifying "clonitis" (too many `.clone()` calls) in the tokenizer or BM25 index builder.
- **Workflow:** Add a `dhat-heap` feature to `Cargo.toml`. When enabled, it outputs a `dhat-heap.json` file viewable in a browser.

### 4. Flamegraphs (`cargo-flamegraph`)
- **Benefit:** Best for identifying where the CPU is spending most of its time (e.g., strictly in `BertModel` vs. string manipulation).
- **Workflow:** Add `just bench flamegraph` which runs the benchmark under `cargo flamegraph`.

### 5. Micro-benchmarking (`criterion`)
- **Candidate Functions:** `tokenize`, `Bm25Index::score`, `dot_product`.
- **Benefit:** Prevents performance regressions in low-level logic.

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | web | Tracing Crate | https://tracing.rs | 2024-01-01 | 2026-03-09 | High | High | Canonical docs for Rust tracing. |
| SRC-02 | web | DHAT-rs | https://github.com/nnethercote/dhat-rs | 2024-01-01 | 2026-03-09 | High | High | Guide for heap profiling. |
| SRC-03 | web | Criterion | https://bheisler.github.io/criterion.rs/book/ | 2024-01-01 | 2026-03-09 | High | High | Standard micro-benchmarking docs. |
