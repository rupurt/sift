# Instrumentation Foundation - Software Design Description

> Implement the Telemetry module, tracing integration, and benchmarking recipes.

## Architecture

### 1. Telemetry Module (`src/system.rs` or `src/search/domain.rs`)
A new `Telemetry` struct using `AtomicUsize` counters to track pipeline events. This is passed into `load_search_corpus`.

```rust
pub struct Telemetry {
    pub heuristic_hits: AtomicUsize,
    pub blob_hits: AtomicUsize,
    pub embedding_hits: AtomicUsize,
    pub total_files: AtomicUsize,
    pub total_segments: AtomicUsize,
}
```

### 2. Tracing Integration
- **Crates:** `tracing`, `tracing-subscriber`.
- **Initialization:** In `main.rs`, initialize a subscriber based on the `verbose` flag count.
  - 0: Off.
  - 1: Info (Spans).
  - 2: Debug (Events).
  - 3: Trace.

### 3. Benchmarking & Profiling
- **Micro-benchmarks:** Add `benches/search_bench.rs` using `criterion`.
- **Flamegraphs:** Add `just bench-flamegraph` calling `cargo flamegraph -- bench latency ...`.
- **Allocation:** Add `dhat-rs` as a dev-dependency.

## Implementation Details

The `SearchEnvironment` will hold an `Arc<Telemetry>` to allow shared access across search runs in a benchmark.
