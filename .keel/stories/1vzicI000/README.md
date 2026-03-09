---
id: 1vzicI000
title: Establish Criterion And Flamegraph Benchmarks
type: feat
status: done
scope: 1vziaX000/1vzibo000
created_at: 2026-03-09T13:30:00
updated_at: 2026-03-09T15:11:17
started_at: 2026-03-09T15:11:17
submitted_at: 2026-03-09T15:11:16
completed_at: 2026-03-09T15:11:17
---

# Establish Criterion And Flamegraph Benchmarks

## Context

Establish long-term performance guardrails with micro-benchmarks and easy profiling.

## Acceptance Criteria

 - [x] [SRS-12/AC-01] Add `criterion` as a dev-dependency and create `benches/search_bench.rs`. <!-- verify: manual, SRS-12:start:end, proof: ac-1.log -->
 - [x] [SRS-12/AC-02] Implement micro-benchmark for `tokenize`. <!-- verify: manual, SRS-12:start:end, proof: ac-2.log -->
 - [x] [SRS-13/AC-01] Add `just bench-flamegraph` recipe. <!-- verify: manual, SRS-13:start:end, proof: ac-3.log -->
