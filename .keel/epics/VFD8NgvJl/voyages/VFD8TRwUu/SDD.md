# Add Merge and Prune Execution Semantics - Software Design Description

> Support branch selection, merge, and prune decisions without forking the
> shared retrieval substrate.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends the frontier runtime with explicit merge and prune behavior.
The runtime should make branch closure, selection, and merged evidence outcomes
replayable rather than letting branches disappear as implicit side effects.

## Context & Boundaries

The first runtime voyage adds branch-local execution and resume state. This
voyage adds the graph-specific operations that make bounded branching useful.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Merge semantics | Merge retained evidence and explicit branch status through recorded graph operations | Keeps branch convergence inspectable |
| Prune semantics | Record prune as a first-class closure outcome | Avoids hidden frontier mutations |
| Runtime layering | Extend the frontier runtime rather than creating a second branch executor | Preserves additive architecture |

## Components

- **Frontier selector**
  Purpose: choose which branch advances next after merge or prune operations.
- **Merge reducer**
  Purpose: combine selected branch outcomes into replayable graph state.
- **Prune recorder**
  Purpose: capture explicit branch closure reasons and trace records.

## Data Flow

1. Accept a graph runtime state with active branches.
2. Apply an explicit select, merge, or prune decision.
3. Update branch status and retained evidence.
4. Record the transition in the graph trace.
5. Return the updated frontier state.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Merge references incompatible branches | Branch state cannot be reconciled | Reject the operation explicitly | Tighten graph transition validation |
| Prune removes an unknown or closed branch | Runtime cannot resolve the target | Fail with explicit contract error | Fix planner or trace emission before retrying |
