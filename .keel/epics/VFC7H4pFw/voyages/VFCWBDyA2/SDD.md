# Benchmark Autonomous Planning Against Baselines - Software Design Description

> Measure autonomous planner quality, stop behavior, and turn efficiency against collapsed single-turn and planned-controller baselines.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends the existing evaluation harness so autonomous planner runs
can be compared directly against the current alternatives. Reports remain
focused on retrieval, evidence, and planning efficiency rather than answer
synthesis.

## Context & Boundaries

The repository already contains planned-controller evaluation and comparative
agentic reporting. This voyage adds autonomous runs, baseline comparisons, and
strategy-aware planner metrics.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────────┐ ┌─────────────┐       │
│  │ Eval        │ │ Comparison  │       │
│  │ Harness     │→│ Reports     │       │
│  └─────────────┘ └─────────────┘       │
└─────────────────────────────────────────┘
        ↑                   ↑
 [Planner Runtime]   [Existing Baselines]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/eval.rs` | internal module | Existing evaluation harness and report shaping | current trunk |
| built-in autonomous runtime | internal runtime | Executes autonomous planner runs for evaluation | current trunk |
| planned-controller evaluation path | existing baseline | Baseline for comparison | current trunk |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Baselines | Compare against collapsed single-turn and planned-controller | Both are current product alternatives |
| Metrics | Focus on retrieval quality, latency, turn count, stop reasons, and evidence efficiency | Matches the mission scope without answer synthesis |
| Artifact stability | Keep reports deterministic and file-backed | Needed for regression review and future tuning |

## Architecture

The evaluation harness adds autonomous execution branches and report fields
without forking the existing planned-controller or direct-search comparison
paths.

## Components

- **Autonomous eval runner**
  Purpose: invoke planner-driven search under the same local workflow as other
  evaluations.
- **Baseline comparator**
  Purpose: align autonomous runs with single-turn and controller baselines.
- **Report formatter**
  Purpose: emit stable planner-aware comparison artifacts.

## Interfaces

The voyage extends existing eval report contracts and CLI-facing summary shapes
rather than adding a second autonomous benchmark command family.

## Data Flow

1. Load the evaluation task set.
2. Run collapsed single-turn, planned-controller, and autonomous variants.
3. Collect strategy-aware planner metrics for the autonomous run.
4. Produce stable comparison artifacts for replay and regression review.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Autonomous evaluation drifts from baselines | Missing comparable task artifacts | Fail the comparison report | Align the harness inputs before retrying |
| Planner metrics are incomplete | Missing stop reason or turn counts | Treat the report as invalid | Extend the autonomous report contract first |
| Report artifacts are unstable | Deterministic tests or replay review fail | Hold rollout of evaluation claims | Tighten serialization and sorting rules |
