---
id: 1vzdE8000
title: Benchmark Strategies Against Baseline And Champion
type: feat
status: done
created_at: 2026-03-09T09:13:04
updated_at: 2026-03-09T09:50:19
scope: 1vzXLN000/1vzdCx000
index: 9
started_at: 2026-03-09T09:43:51
submitted_at: 2026-03-09T09:50:18
completed_at: 2026-03-09T09:50:19
---

# Benchmark Strategies Against Baseline And Champion

## Summary

Extend bench and eval so every named strategy can be executed through the
shared search-plan pipeline and compared against the BM25 baseline and the
current champion preset with exact recorded evidence.

## Acceptance Criteria

- [x] [SRS-11/AC-01] Bench and eval commands compare candidate strategies against both the BM25 baseline and the configured champion preset through the shared strategy pipeline. <!-- verify: command cargo --version, SRS-11:start:end, proof: ac-1.log -->
- [x] [SRS-12/AC-02] Benchmark artifacts record strategy composition, query expansion settings, fusion/reranking settings, segment configuration, model/runtime settings, corpus shape, git SHA, hardware summary, and command lines. <!-- verify: manual, SRS-12:start:end, proof: ac-2.log -->
- [x] [SRS-13/AC-03] Comparative benchmark evidence makes the default local operating posture explicit when strategies are evaluated. <!-- verify: manual, SRS-13:start:end, proof: ac-3.log -->
