---
id: 1vzJfc000
title: Build SciFact Evaluation And Benchmark Harness
type: feat
scope: 1vzJVa000/1vzJda000
status: backlog
created_at: 2026-03-08T12:20:08
updated_at: 2026-03-08T12:24:04
---

# Build SciFact Evaluation And Benchmark Harness

## Summary

Create the evaluation and benchmark foundation for the indexless MVP by adding
SciFact corpus download/materialization commands plus BM25-oriented quality and
latency benchmark commands that emit reproducible evidence.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `sift eval download scifact` and `sift eval materialize scifact` produce a local benchmark workspace containing stable document IDs, UTF-8 text files, query data, and qrels suitable for later CLI evaluation. <!-- verify: cargo test eval:: + cargo run -- eval download scifact --out .cache/eval/scifact + cargo run -- eval materialize scifact --source .cache/eval/scifact --out .cache/eval/scifact-files, SRS-01:start:end, proof: ac-1.log -->
- [ ] [SRS-02/AC-01] `sift bench quality --engine bm25` and `sift bench latency --engine bm25` execute against the materialized corpus and emit structured benchmark output. <!-- verify: cargo test bench:: + cargo run -- bench quality --engine bm25 --corpus .cache/eval/scifact-files --qrels .cache/eval/scifact/qrels/test.tsv + cargo run -- bench latency --engine bm25 --corpus .cache/eval/scifact-files --queries .cache/eval/scifact/test-queries.tsv, SRS-02:start:end, proof: ac-2.log -->
- [ ] [SRS-03/AC-01] Benchmark output records the exact command, git SHA, hardware summary, corpus counts, and measured timing or metric fields needed for reproducible evidence capture. <!-- verify: cargo test bench::report + cargo run -- bench latency --engine bm25 --corpus .cache/eval/scifact-files --queries .cache/eval/scifact/test-queries.tsv, SRS-03:start:end, proof: ac-3.log -->
