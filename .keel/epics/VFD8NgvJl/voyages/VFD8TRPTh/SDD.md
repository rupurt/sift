# Build Frontier Runtime and Branch-Local Evidence - Software Design Description

> Execute bounded graph episodes over a frontier with branch-local retained
> evidence and continuation state.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds the first graph runtime path. It should interpret graph
episode state, execute branch retrieval turns through the existing controller
substrate, and preserve retained evidence per branch without forking the
underlying search engine.

## Context & Boundaries

The graph contract and replay work define the records and validation rules.
This voyage executes graph state but stays short of merge and prune semantics.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Runtime layering | Lower branch retrieval work through the shared controller or search substrate | Keeps graph execution additive to the shipped runtime |
| Evidence model | Keep retained evidence branch-local | Graph branches need local memory without cross-branch contamination |
| Resume model | Persist explicit frontier and branch status | Resumability should come from data, not hidden executor state |

## Components

- **Frontier executor**
  Purpose: advance active branches in bounded order.
- **Branch-local evidence store**
  Purpose: track retained evidence per branch across steps.
- **Graph runtime adapter**
  Purpose: lower graph steps into the shared retrieval and controller substrate.

## Data Flow

1. Accept graph episode state with an active frontier.
2. Select the next bounded branch to execute.
3. Lower the branch’s retrieval work through the shared runtime.
4. Update branch-local retained evidence and frontier state.
5. Return updated graph runtime state.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Branch execution exceeds bounds | Branch or step limits are exceeded | Stop the graph episode explicitly | Preserve bounded runtime behavior |
| Branch-local evidence cannot be resumed | Stored graph state is incomplete or invalid | Fail with explicit runtime error | Repair the graph contract or replay logic first |
