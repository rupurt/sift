---
id: 1vzJfv000
title: Add Candle Dense Reranking And Hybrid Fusion
type: feat
scope: 1vzJVa000/1vzJda000
status: done
created_at: 2026-03-08T12:20:27
updated_at: 2026-03-08T15:22:26
started_at: 2026-03-08T14:25:53
completed_at: 2026-03-08T15:22:26
---

# Add Candle Dense Reranking And Hybrid Fusion

## Summary

Add the dense half of the MVP by loading a local pure-Rust embedding model,
reranking a bounded lexical shortlist, fusing BM25 and dense signals into the
default search path, and proving the quality and latency behavior against the
evaluation corpus.

## Acceptance Criteria

- [x] [SRS-07/AC-01] The default `sift search` path combines BM25 full-corpus retrieval with dense reranking on a bounded shortlist and produces one final hybrid ranking. <!-- verify: sh -lc 'cargo test hybrid::fusion && cargo run -- search "retrieval architecture" .cache/eval/scifact-files --limit 3', SRS-07:start:end, proof: ac-1.log -->
- [x] [SRS-08/AC-01] Dense inference runs through a local pure-Rust runtime and model-loading path rather than a remote API, daemon, or native service dependency. <!-- verify: sh -lc 'cargo test dense::model && cargo tree | rg "candle|rust_tokenizers" && cargo run -- search "retrieval architecture" .cache/eval/scifact-files --limit 3', SRS-08:start:end, proof: ac-2.log -->
- [x] [SRS-09/AC-01] `sift bench quality` compares BM25-only and hybrid runs on the SciFact qrels and records the exact metric delta between them. <!-- verify: sh -lc 'cargo test bench::quality', SRS-09:start:end, proof: ac-3.log -->
- [x] [SRS-10/AC-01] `sift bench latency --engine hybrid` records measured p50, p90, and worst-case latency against the 200 ms target and preserves any shortfall as explicit evidence instead of hiding it. <!-- verify: sh -lc 'cargo test bench::latency', SRS-10:start:end, proof: ac-4.log -->
