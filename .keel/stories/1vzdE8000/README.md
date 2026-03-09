---
id: 1vzdE8000
title: Benchmark Strategies Against Baseline And Champion
type: feat
status: backlog
created_at: 2026-03-09T09:13:04
updated_at: 2026-03-09T09:20:48
scope: 1vzXLN000/1vzdCx000
index: 9
---

# Benchmark Strategies Against Baseline And Champion

## Summary

Extend bench and eval so every named strategy can be executed through the
shared search-plan pipeline and compared against the BM25 baseline and the
current champion preset with exact recorded evidence.

## Acceptance Criteria

- [ ] [SRS-11/AC-01] Bench and eval commands compare candidate strategies
      against both the BM25 baseline and the configured champion preset through
      the shared strategy pipeline.
- [ ] [SRS-12/AC-02] Benchmark artifacts record strategy composition, query
      expansion settings, fusion/reranking settings, segment configuration,
      model/runtime settings, corpus shape, git SHA, hardware summary, and
      command lines.
- [ ] [SRS-13/AC-03] Comparative benchmark evidence makes the default local
      operating posture explicit when strategies are evaluated.
