---
created_at: 2026-03-08T14:10:04
---

# Reflection - Build SciFact Evaluation And Benchmark Harness

## Knowledge

- [1vzLO0001](../../knowledge/1vzLO0001.md) Keep materialized eval corpora self-contained

## Observations

- The story landed cleanly once the scope was kept narrow: download raw SciFact
  assets, materialize them into local `.txt` files, and benchmark only BM25.
- Removing the stale `zvec-sys` dependency from the active build path made the
  new retrieval work much faster to iterate on.
- The BM25 baseline over the full 5,183-document SciFact corpus is already
  usable: the recorded latency proof showed roughly 29.7 ms p50, 45.2 ms p90,
  and 80.1 ms max query time after a ~2.15 s corpus-preparation step on the
  current Linux workstation.
- The most annoying non-code issue was keeping Keel verification annotations
  executable. Replacing placeholder `+` separators with real `sh -lc '... && ...'`
  commands removed friction when recording evidence.
