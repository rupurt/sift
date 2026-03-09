---
id: 1vzSwK000
title: Benchmark True Hybrid Retrieval
type: feat
scope: 1vzSne000/1vzSy6000
status: in-progress
created_at: 2026-03-08T22:14:00
updated_at: 2026-03-09T01:29:34
started_at: 2026-03-09T01:29:34
---

# Benchmark True Hybrid Retrieval

## Summary

Extend the benchmark and evaluation harnesses so they measure the true-hybrid
architecture, record the new vector/segment configuration, and make any latency
tradeoffs explicit.

## Acceptance Criteria

- [x] [SRS-07/AC-01] Benchmark and evaluation commands compare BM25-only retrieval with the true-hybrid path and report the resulting metric deltas. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test bench::quality && cargo run -- bench quality --engine hybrid --baseline bm25 --corpus tests/fixtures/rich-docs --queries tests/fixtures/rich-docs/test-queries.tsv --qrels tests/fixtures/rich-docs/qrels/test.tsv', SRS-07:start:end, proof: ac-1.log -->
- [x] [SRS-08/AC-01] Benchmark output records the segment configuration, embedding model settings, command line, git SHA, corpus shape, and hardware summary for the true-hybrid path. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test bench::metadata && cargo run -- bench latency --engine hybrid --corpus tests/fixtures/rich-docs --queries tests/fixtures/rich-docs/test-queries.tsv', SRS-08:start:end, proof: ac-2.log -->
- [x] [SRS-09/AC-01] The true-hybrid implementation does not create or require a persisted vector sidecar index or background service. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! find . -path "./target" -prune -o -path "./.git" -prune -o -name "*.idx" -o -name "*.faiss" -o -name "*.ann" -o -name "*.hnsw" -print | rg . && ./target/release/sift search tests/fixtures/rich-docs "semantic retrieval" --engine hybrid', SRS-09:start:end, proof: ac-3.log -->
- [x] [SRS-10/AC-01] The default vector retrieval runtime remains the local pure-Rust Candle path rather than introducing `fastembed-rs` or ONNX Runtime as the default dependency. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo tree | rg "candle" && ! cargo tree | rg " fastembed|\\bort\\b"', SRS-10:start:end, proof: ac-4.log -->
- [x] [SRS-11/AC-01] Latency reporting makes any shortfall against the 200 ms target explicit for the true-hybrid path. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test bench::latency && cargo run -- bench latency --engine hybrid --corpus tests/fixtures/rich-docs --queries tests/fixtures/rich-docs/test-queries.tsv', SRS-11:start:end, proof: ac-5.log -->
