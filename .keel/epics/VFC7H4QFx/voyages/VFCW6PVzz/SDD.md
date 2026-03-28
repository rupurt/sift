# Ship Heuristic Planner Baseline - Software Design Description

> Generate bounded autonomous turns from a root task using a deterministic heuristic planner over retained evidence and the existing hybrid substrate.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces the first built-in planner policy: a deterministic
heuristic planner that can turn a root task into search decisions without
caller-authored turn lists. The policy stays linear-first and emits explicit
stop reasons so later runtime and evaluation work can reuse one planner shape.

## Context & Boundaries

The contract voyage already shipped the planner DTOs, strategy selection
records, and execution seam. This voyage fills the heuristic policy slot only.

### In Scope

- initial query generation from the root task
- follow-up query derivation from retained evidence
- explicit heuristic stop behavior

### Out of Scope

- model-driven planning
- CLI surface work
- branching execution

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────────┐ ┌─────────────┐      │
│  │ Root Task   │ │ Heuristic   │      │
│  │ + Evidence  │→│ Planner     │      │
│  └─────────────┘ └─────────────┘      │
└─────────────────────────────────────────┘
        ↑                    ↑
 [Planner DTOs]      [Autonomous Seam]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/search/domain.rs` | internal module | Planner DTOs and decision/trace contracts | current trunk |
| `AutonomousPlanner` trait | supported contract | Shared policy interface for heuristic and model-driven planning | current trunk |
| `src/facade.rs` | internal module | Hosts the library autonomous seam that will call this policy | current trunk |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Query generation | Start with deterministic lexical decomposition of the root task | Gives the first planner a stable baseline without model coupling |
| Follow-up refinement | Derive next queries from retained artifact snippets and prior planner output | Reuses the existing evidence substrate rather than inventing external state |
| Stop model | Emit explicit stop reasons instead of relying on empty loops or logs | Keeps bounded planning replayable and testable |

## Architecture

The heuristic planner sits above the autonomous DTO layer and below the
built-in runtime. It consumes `AutonomousSearchRequest` and returns an
`AutonomousPlannerTrace` composed of `Search`, `Continue`, and `Terminate`
decisions.

## Components

- **Heuristic planner policy**
  Purpose: map a root task and retained evidence into deterministic search
  decisions.
- **Query refinement logic**
  Purpose: generate follow-up queries without repeating prior search work.
- **Stop evaluator**
  Purpose: decide when bounded linear planning should terminate and record why.

## Interfaces

The voyage implements the existing `AutonomousPlanner` contract only. It does
not expose a new public graph interface or a second planner DTO family.

## Data Flow

1. Accept an `AutonomousSearchRequest`.
2. Generate the first search decision from the root task.
3. Inspect retained evidence to derive the next query, if any.
4. Emit `Continue` or `Terminate` with an explicit stop reason.
5. Return the completed planner trace to the autonomous runtime.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| No productive query can be derived | Heuristic refinement yields no new query | Emit `Terminate` with `NoFurtherQueries` | Let higher layers inspect the planner trace |
| Evidence is exhausted | Retained evidence yields no new signal | Emit `Terminate` with `NoAdditionalEvidence` | Stop cleanly without synthetic work |
| Step budget is exhausted | Current step reaches the configured limit | Emit `Terminate` with `StepLimitReached` | Preserve bounded linear behavior |
