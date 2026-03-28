# Define Planner State and Stop Semantics - Software Design Description

> Make planner decisions, continuation criteria, and linear stop semantics explicit and replayable while keeping the design extensible to future branching search.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage defines the contract layer for autonomous planning before the
planner policy is implemented. It introduces planner-facing DTOs, strategy
selection, replayable stop semantics, and a library-first execution seam that
can sit above the current retrieval/controller runtime.

## Context & Boundaries

The current runtime can execute explicit planned turns through
`SearchControllerRequest`, but it cannot represent an autonomous planning
episode that begins with a root task and ends with planner-generated turns and
stop reasons. This voyage addresses the contract and runtime seam only.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌──────────────┐ ┌──────────────┐     │
│  │ Planner DTOs │ │ Planner Seam │     │
│  └──────────────┘ └──────────────┘     │
└─────────────────────────────────────────┘
        ↑               ↑
   [Facade]       [Current Controller]
```

Out of scope for this voyage:
- autonomous turn-generation policy
- CLI agent flag support
- branching or graph-search execution

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/search/domain.rs` | internal module | Home for planner-facing DTOs and state records | current trunk |
| `src/facade.rs` | internal module | Current library execution seam that hosts single-turn and planned-controller paths | current trunk |
| `SearchControllerRequest` / `SearchControllerResponse` | internal contract | Existing deterministic multi-turn runtime to compose with planner output | current trunk |
| `VDzNOMF7T` | accepted ADR | Modular engine and execution seams | current trunk |
| `VDzNPELXy` | accepted ADR | Explicit turn protocol and inspectable emissions | current trunk |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Planner form | Introduce planner DTOs alongside current controller DTOs | Avoids overloading the planned-controller contract with autonomous concerns |
| Strategy selection | Make planner strategy explicit data | Supports heuristic and model-driven planning behind one seam |
| Stop semantics | Record explicit stop reasons and planner decisions | Replayable autonomy requires inspectable termination |
| Growth path | Stay linear-first but keep stable identifiers and reason codes | Preserves an upgrade path toward future branching search |
| Surface | Library-first seam now, CLI later | Matches the approved mission scope and lowers rollout risk |

## Architecture

The planner layer sits one step above the current deterministic controller:

- planner request/response/state define the autonomous episode contract
- planner strategy selects how turns will eventually be proposed
- planner decisions and stop reasons explain why the episode continued or stopped
- autonomous execution composes planner output with the current retrieval and
  controller runtime rather than replacing it

The existing `SearchControllerRequest` remains the substrate for replaying an
explicit turn sequence. The new planner seam should be able to lower into that
runtime while keeping planner state visible at the outer layer.

## Components

- **Planner request/response types**
  Purpose: represent a root task, planner strategy, retained evidence budget,
  and final planner-aware response.
- **Planner state**
  Purpose: track current step, retained evidence, completion, and planner-local
  status needed to resume or replay an autonomous run.
- **Planner decision and stop-reason records**
  Purpose: explain continuation, pruning, and termination as explicit data.
- **Autonomous execution seam**
  Purpose: provide a library-first entry point that can later call real planner
  policies while composing with the current controller runtime.

## Interfaces

Expected contract shape:

- planner request:
  root task, planner strategy, shared search plan, turn limit, retained-artifact
  limit, and emission mode
- planner state:
  current step, retained artifacts, completion flag, and planner-local status
- planner decision:
  action, rationale, optional proposed query, optional stop reason
- planner response:
  planner state, planner trace, and any lowered controller/search responses

The voyage should keep these interfaces library-first and avoid exposing a
generic public search graph.

## Data Flow

1. Caller submits an autonomous planner request with a root task.
2. Runtime initializes planner state and strategy selection.
3. Planner layer emits explicit planner decisions and stop semantics.
4. Planner output lowers into the existing controller/search runtime when
   explicit turns exist.
5. Response returns planner state and replayable planner traces at the outer
   layer.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Planner contract is ambiguous | API review or test drift | Refuse to stabilize the seam | Tighten DTO boundaries before planner policy work |
| Stop semantics are not replayable | Missing explicit stop reason in tests | Treat as contract failure | Add stop-reason records before shipping |
| Planner seam regresses current controller path | Regression proof fails | Keep autonomous path additive and disabled by default | Revisit seam layering |
