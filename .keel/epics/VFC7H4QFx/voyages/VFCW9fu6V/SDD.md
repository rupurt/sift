# Add Model-Driven Planner Strategy - Software Design Description

> Introduce a local-first model-driven planner strategy that reuses the shared autonomous planner contract and bounded linear runtime.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends the shared planner contract with a model-driven policy
implementation and runtime selection path. It keeps the heuristic planner as
the baseline and makes model-driven planning an additive strategy choice.

## Context & Boundaries

The contract layer and built-in autonomous runtime already exist. This voyage
adds the first model-backed planner implementation under the same contracts and
selection model.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────────┐ ┌─────────────┐       │
│  │ Strategy    │ │ Model       │       │
│  │ Resolver    │→│ Planner     │       │
│  └─────────────┘ └─────────────┘       │
└─────────────────────────────────────────┘
        ↑                   ↑
 [Autonomous Request] [Local Model Runtime]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `AutonomousPlannerStrategy` | supported contract | Strategy selection and profile routing | current trunk |
| `Sift::generative` / local model runtime | supported/internal runtime | Local generative model access for planner reasoning | current trunk |
| built-in autonomous runtime | internal module | Execution path that consumes the selected planner policy | current trunk |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Strategy model | Reuse `AutonomousPlannerStrategy` kind/profile instead of adding a second selector | Keeps heuristic and model-driven planning on one contract |
| Model locality | Use local in-process generation only | Preserves the local-first product thesis |
| Failure mode | Return explicit unavailable-profile errors | Keeps heuristic fallback and evaluation semantics inspectable |

## Architecture

The runtime resolves `AutonomousPlannerStrategyKind::ModelDriven` into a local
planner adapter that emits the same decision/trace types as the heuristic
planner. The autonomous executor remains unchanged aside from strategy
resolution.

## Components

- **Model-driven planner adapter**
  Purpose: convert root task and retained evidence into planner decisions using
  a local model.
- **Strategy resolver**
  Purpose: pick heuristic or model-driven planning from request strategy data.
- **Profile guard**
  Purpose: validate requested planner profiles and surface explicit errors.

## Interfaces

No new planner DTO family is introduced. The voyage extends the behavior behind
the existing planner contracts and runtime surface only.

## Data Flow

1. Accept an autonomous request with planner strategy data.
2. Resolve the requested planner implementation.
3. Run the model-driven planner adapter locally.
4. Emit a planner trace under the shared contract.
5. Hand the trace to the shared autonomous runtime.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Requested planner profile is unavailable | Strategy profile lookup fails | Return explicit runtime error | Fall back to heuristic only when the caller requests it |
| Local model generation fails | Model runtime returns an error | Fail the autonomous run without hidden fallback | Inspect the planner/runtime error directly |
| Model-driven output is unbounded | Decisions exceed step budget or omit stop semantics | Reject the trace as invalid | Preserve bounded linear guarantees |
