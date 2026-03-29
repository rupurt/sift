# Benchmark Graph Search Against Linear Baselines - Software Design Description

> Measure graph planner performance against linear autonomy, planned-controller,
> and single-turn baselines.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends the current autonomous evaluation harness with graph-aware
fixtures, metrics, and report shapes. The report should make graph and linear
planner tradeoffs visible rather than treating graph search as just another
linear run.

## Context & Boundaries

The repo already benchmarks linear autonomous runs against planned-controller
and collapsed baselines. This voyage keeps that harness and adds graph-aware
reporting.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Baseline set | Compare graph search against linear autonomy, planned-controller, and collapsed single-turn | Keeps graph value grounded in current shipped behavior |
| Metrics shape | Add graph-specific metrics alongside existing success or recall metrics | Graph search needs graph-specific signal |
| Artifact shape | Keep reports replayable and regression-friendly | Mission verification depends on inspectable evidence |

## Components

- **Graph fixture harness**
  Purpose: execute graph and linear runs on shared tasks.
- **Graph metrics aggregator**
  Purpose: calculate branch and frontier-specific metrics.
- **Comparative report emitter**
  Purpose: serialize graph-versus-linear evidence for regression review.

## Data Flow

1. Load shared evaluation fixtures.
2. Execute graph, linear autonomous, planned-controller, and baseline runs.
3. Aggregate graph-specific and shared metrics.
4. Emit a comparative report artifact.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Graph fixture cannot replay deterministically | Report generation diverges across repeated runs | Fail evaluation rather than emit misleading evidence | Tighten graph fixture or runtime determinism |
| Graph metrics are unavailable for a run | Runtime does not emit required graph trace data | Reject the report as incomplete | Backfill graph trace fields before rerunning |
