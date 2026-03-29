# Define Graph Episode Contracts - Software Design Description

> Introduce graph episode request, state, response, node, and edge records that
> make bounded branching search explicit and replayable.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage formalizes the graph-shaped data model that the rest of the mission
will execute. The design extends the current autonomous DTO family with graph
episode state, node or edge references, and explicit frontier membership while
preserving the existing linear path.

## Context & Boundaries

The current codebase already has linear planner state, flat trace steps, and a
controller-based runtime seam. This voyage adds graph records only; it does not
yet execute or generate graph behavior.

### In Scope

- graph episode request, response, and state DTOs
- graph node, edge, and branch-status records
- additive graph-mode selection on the autonomous surface

### Out of Scope

- frontier execution
- graph planner policy
- CLI controls

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Contract style | Extend the current autonomous contract rather than replacing it | Keeps graph search additive to the shipped linear surface |
| Identity model | Use explicit graph node or branch identifiers and edge references | Replay and validation need stable graph-local ids |
| Episode state | Record frontier membership and branch status as data | Later runtime work should not infer branch state from logs |

## Architecture

The voyage adds a graph episode layer beside the current linear planner DTOs.
Graph requests and responses should be able to point at the same underlying
search substrate later without prematurely choosing execution policy.

## Components

- **Graph episode records**
  Purpose: represent graph-mode requests, state, and responses.
- **Graph topology records**
  Purpose: represent graph nodes, edges, and parent or child relationships.
- **Graph mode selector**
  Purpose: keep graph behavior additive to the existing autonomous surface.

## Data Flow

1. Accept a graph-aware autonomous request.
2. Persist graph episode state, including frontier and branch status.
3. Record graph nodes and edges with stable ids.
4. Return a graph-aware response or trace record without executing it yet.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Graph episode ids are missing or unstable | Contract review or serialization tests fail | Reject the DTO shape before runtime work lands | Tighten record constructors and tests |
| Graph mode conflicts with current linear request semantics | Compatibility review fails | Keep graph mode additive instead of replacing linear records | Adjust request layering before runtime work |
