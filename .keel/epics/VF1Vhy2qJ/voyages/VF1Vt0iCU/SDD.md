# Build Local Multi-Turn Loop Execution - Software Design Description

> Implement a bounded local controller that decomposes queries, reuses the existing hybrid substrate across turns, manages context, and terminates deterministically.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds the first formal local controller on top of the existing hybrid substrate. The controller should operate as an explicit loop over turn state: generate or select the next retrieval action, execute against the current substrate, update retained evidence within a bounded context, and decide whether to continue or stop.

## Context & Boundaries

The voyage is responsible for execution semantics, not for inventing a new retrieval stack. Retrieval, fusion, reranking, and caching should remain the substrate. The controller orchestrates them; it does not replace them.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌────────────┐ ┌────────────┐ ┌──────┐│
│  │Controller  │ │LoopExecution│ │State ││
│  └────────────┘ └────────────┘ └──────┘│
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `SearchEnvironment` | internal module | Reusable prepared-corpus search execution | current trunk |
| `SearchService` | internal module | Hybrid retrieval substrate | current trunk |
| `GenerativeModel` / `Conversation` | internal trait | Optional local model support for decomposition or controller logic | current trunk |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Execution seam | Add a loop-oriented execution path rather than mutating the current pipeline into implicit recursion | Keeps single-turn and multi-turn execution distinguishable |
| Plan authority | Reduce implicit runtime overrides and move controller choices into explicit state/plan data | Agentic behavior must be inspectable |
| Context management | Start with deterministic bounded heuristics before learned policies | Keeps the first controller local and testable |

## Architecture

The controller layer sits above the hybrid substrate:
- controller state chooses the next retrieval action
- loop execution invokes the existing prepared corpus environment
- retained evidence is updated under bounded rules
- termination produces a completed turn trace and final emission

## Components

- Controller state machine: owns turn index, retained evidence, stop conditions, and next-step selection.
- Loop execution adapter: repeatedly calls the substrate using explicit state.
- Context manager: retains, discards, or reorders evidence under bounded budgets.
- Invocation surfaces: CLI and/or facade methods that select this execution path.

## Interfaces

- The controller should accept a turn-aware request and return a trace-capable response.
- Single-turn search should remain available as the current default path.

## Data Flow

1. Build controller state from a turn-aware request.
2. Generate or select the next retrieval query.
3. Execute retrieval through the existing hybrid substrate.
4. Update retained evidence and trace records.
5. Decide whether to stop or continue.
6. Emit the final response and trace.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Controller loops indefinitely | Turn limit or stop condition violation | Abort with explicit error/trace state | Tune termination logic |
| Context budget overflows | Budget check fails | Prune or reject additional evidence | Continue with bounded context |
| Single-turn path regresses | Test or proof failure | Keep agentic path additive until stable | Revisit execution split |
