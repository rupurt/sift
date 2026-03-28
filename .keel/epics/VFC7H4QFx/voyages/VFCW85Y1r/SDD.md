# Wire Strategy-Selected Autonomous Runtime - Software Design Description

> Route autonomous planner strategy selection through a built-in runtime that remains bounded, linear, and additive to the current single-turn and controller paths.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the planner seam into an internal runtime path that can run a
built-in planner policy end to end. It lowers planner-issued search decisions
into the existing multi-turn controller substrate instead of introducing a
parallel autonomous executor.

## Context & Boundaries

The heuristic policy voyage provides planner decisions; this voyage executes
them. The model-driven strategy voyage will extend the same runtime rather than
replacing it.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌──────────────┐ ┌──────────────┐     │
│  │ Planner      │ │ Shared       │     │
│  │ Runtime      │→│ Controller   │     │
│  └──────────────┘ └──────────────┘     │
└─────────────────────────────────────────┘
        ↑                  ↑
 [Planner Policy]   [Search Runtime]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/facade.rs` | internal module | Home for the built-in autonomous runtime path | current trunk |
| `SearchControllerRequest` | supported contract | Shared multi-turn execution substrate | current trunk |
| `AutonomousPlannerTrace` | supported contract | Planner-side execution trace and state progression | current trunk |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Runtime layering | Lower planner search decisions into `SearchControllerRequest` | Reuses existing retained-evidence and trace semantics |
| Resume model | Advance through explicit planner state/current step | Keeps bounded episodes resumable and replayable |
| Additive rollout | Keep autonomous behavior behind a separate runtime entry point | Prevents regression to the stable non-autonomous paths |

## Architecture

The runtime receives an `AutonomousSearchRequest`, resolves the built-in
planner policy, converts planner `Search` decisions into turn requests, and
invokes the existing controller runtime for actual retrieval execution.

## Components

- **Autonomous runtime entry point**
  Purpose: own planner resolution and controller lowering.
- **Turn-lowering adapter**
  Purpose: convert planner decisions into `SearchTurnRequest` values.
- **State merger**
  Purpose: combine planner state and controller retention back into one
  autonomous response.

## Interfaces

The voyage adds a built-in autonomous runtime surface that remains aligned with
the existing crate-root planner DTOs and shared controller contracts.

## Data Flow

1. Accept an autonomous request.
2. Resolve the built-in planner policy and collect the planner trace.
3. Lower each planner `Search` decision into a turn request.
4. Execute those turns through the shared controller runtime.
5. Merge retained evidence, planner trace, and controller trace into the final
   autonomous response.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Planner trace omits a query for a search decision | Search decision has no query payload | Fail the autonomous run with a contract error | Fix planner policy before retrying |
| Planner trace exceeds the step limit | Planner step count outruns request state | Reject the run as unbounded | Preserve bounded linear semantics |
| Runtime drift from controller behavior | Regression tests fail or traces diverge | Keep the autonomous runtime additive and inspectable | Reuse shared controller paths only |
