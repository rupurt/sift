# Comprehensive Performance Instrumentation — Brief

## Hypothesis

By integrating structured telemetry (spans and events), allocation profiling, and cache efficiency metrics, we can transform performance optimization from "guesswork based on wall-clock time" into a data-driven process. This will allow us to identify precise bottlenecks in the search asset pipeline and validate the impact of future optimizations (like parallelization or I/O changes).

## Problem Space

As `sift` scales to larger repositories (thousands of files), the complex interactions between filesystem I/O, BLAKE3 hashing, document extraction, and neural network inference become difficult to debug. We have experienced "stuck" states and high latency without a clear view of where the time is spent or where thread contention occurs.

## Success Criteria

- [ ] Implement a `Telemetry` module to track and display cache hit rates (heuristic, blob, embedding).
- [ ] Integrate the `tracing` crate to provide structured spans for all major search phases.
- [ ] Establish a micro-benchmarking harness using `criterion` for hot-path functions.
- [ ] Integrate allocation profiling (e.g., `dhat`) into the development workflow.
- [ ] Provide a `just` recipe for generating flamegraphs to identify compute bottlenecks.
- [ ] Ensure all instrumentation is lightweight and does not significantly degrade baseline performance.

## Open Questions

- Should cache hit rates be a permanent part of the benchmark report table or only visible at `-v`?
- How do we handle `tracing` output for CLI users? (e.g., a `--trace-json` flag for external viewers?)
- Which functions are the highest priority for `criterion` micro-benchmarks?
