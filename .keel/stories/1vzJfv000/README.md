---
id: 1vzJfv000
title: Add Candle Dense Reranking And Hybrid Fusion
type: feat
status: icebox
created_at: 2026-03-08T12:20:27
updated_at: 2026-03-08T12:20:27
---

# Add Candle Dense Reranking And Hybrid Fusion

## Summary

Add the dense half of the MVP by loading a local pure-Rust embedding model,
reranking a bounded lexical shortlist, fusing BM25 and dense signals into the
default search path, and proving the quality and latency behavior against the
evaluation corpus.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] The default `sift search` path combines BM25 full-corpus retrieval with dense reranking on a bounded shortlist and produces one final hybrid ranking. <!-- verify: cargo test hybrid::fusion + cargo run -- search "retrieval architecture" .cache/eval/scifact-files, SRS-03:start:end, proof: ac-1.log -->
- [ ] [SRS-NFR-01/AC-01] Dense inference runs through a local pure-Rust runtime and model-loading path rather than a remote API, daemon, or native service dependency. <!-- verify: cargo test dense::model + cargo tree + cargo run -- search "retrieval architecture" .cache/eval/scifact-files, SRS-NFR-01:start:end, proof: ac-2.log -->
- [ ] [SRS-04/AC-02] `sift bench quality` compares BM25-only and hybrid runs on the SciFact qrels and records the exact metric delta between them. <!-- verify: cargo test bench::quality + cargo run -- bench quality --engine hybrid --baseline bm25 --corpus .cache/eval/scifact-files --qrels .cache/eval/scifact/qrels/test.tsv, SRS-04:start:end, proof: ac-3.log -->
- [ ] [SRS-NFR-03/AC-01] `sift bench latency --engine hybrid` records measured p50, p90, and worst-case latency against the 200 ms target and preserves any shortfall as explicit evidence instead of hiding it. <!-- verify: cargo test bench::latency + cargo run -- bench latency --engine hybrid --corpus .cache/eval/scifact-files --queries .cache/eval/scifact/test-queries.tsv, SRS-NFR-03:start:end, proof: ac-4.log -->
