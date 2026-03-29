# Add Model-Driven Graph Planner Strategy - Software Design Description

> Reuse the graph planner contract with a local model-driven planner that can
> propose branching decisions.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends the shared graph planner contract with a model-driven
policy. It keeps the heuristic graph planner as the baseline and makes
model-driven branching an additive strategy choice.

## Context & Boundaries

The graph contract, frontier runtime, and heuristic graph planner should
already exist. This voyage only adds the first model-backed graph planning
strategy.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Strategy selector | Reuse the existing planner strategy concept for graph mode | Keeps heuristic and model-driven graph planning on one contract |
| Model locality | Use local in-process generation only | Preserves the local-first product thesis |
| Failure mode | Return explicit unavailable-profile errors | Avoids hidden fallback that would blur evaluation |

## Components

- **Model-driven graph planner adapter**
  Purpose: translate root task and graph state into graph decisions using a local model.
- **Strategy resolver**
  Purpose: choose heuristic or model-driven graph planning from request strategy data.
- **Profile guard**
  Purpose: validate requested graph planner profiles and surface explicit errors.

## Data Flow

1. Accept a graph search request with model-driven planner strategy data.
2. Resolve the local graph planner profile.
3. Run the model-driven graph planner adapter.
4. Emit a graph trace under the shared graph contract.
5. Hand the trace to the graph runtime unchanged.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Requested graph planner profile is unavailable | Strategy profile lookup fails | Return explicit runtime error | Fall back only when the caller requests it |
| Local model generation fails | Model runtime returns an error | Fail the graph run without hidden fallback | Inspect the planner/runtime error directly |
