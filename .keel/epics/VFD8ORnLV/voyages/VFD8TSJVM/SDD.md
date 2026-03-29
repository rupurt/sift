# Ship Heuristic Graph Planner Baseline - Software Design Description

> Introduce a deterministic graph planner that can fork and prioritize a
> bounded frontier without model dependency.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces the first graph planner policy. The heuristic baseline
must emit graph decisions that create and prioritize a bounded frontier while
remaining deterministic and local-first.

## Context & Boundaries

The graph contract and frontier runtime voyages provide the data model and
executor. This voyage fills the heuristic graph-policy slot only.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Branch seeding | Fork from root-task decomposition and retained evidence | Gives the baseline graph planner a deterministic local policy |
| Frontier priority | Use explicit heuristic scoring rather than hidden ordering | Keeps graph behavior inspectable |
| Stop model | Emit explicit terminate reasons rather than empty frontiers alone | Preserves replayability and boundedness |

## Components

- **Heuristic graph planner**
  Purpose: emit graph decisions from task and branch-local evidence.
- **Frontier priority policy**
  Purpose: rank which branch or node should advance next.
- **Stop evaluator**
  Purpose: terminate bounded graph episodes with explicit reasons.

## Data Flow

1. Accept graph episode state and retained evidence.
2. Seed or update the active frontier.
3. Emit fork or select decisions deterministically.
4. Emit continue or terminate decisions with bounded reasons.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Heuristic policy exceeds graph bounds | Branch or step limits are exceeded | Terminate with an explicit bounded reason | Preserve graph runtime limits |
| Frontier produces no productive next branch | Heuristic scoring yields no useful successor | Terminate with explicit stop reason | Let evaluation compare the baseline honestly |
