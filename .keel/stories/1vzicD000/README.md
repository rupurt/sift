---
id: 1vzicD000
title: Instrument Search Pipeline And Report Metrics
type: feat
status: in-progress
scope: 1vziaX000/1vzibo000
created_at: 2026-03-09T13:30:00
updated_at: 2026-03-09T15:00:21
started_at: 2026-03-09T15:00:21
---

# Instrument Search Pipeline And Report Metrics

## Context

Use the telemetry and tracing models to instrument the actual search path and report cache effectiveness.

## Acceptance Criteria

- [ ] [SRS-11/AC-01] Wrap `execute`, `load_search_corpus`, and `score_segments` in `tracing` spans. <!-- verify: manual, SRS-11:start:end, proof: ac-1.log -->
- [ ] [SRS-09/AC-03] Increment `Telemetry` counters on cache hits/misses. <!-- verify: manual, SRS-09:start:end, proof: ac-2.log -->
- [ ] [SRS-10/AC-01] Display cache hit rates in the benchmark summary table. <!-- verify: manual, SRS-10:start:end, proof: ac-3.log -->
