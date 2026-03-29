# Graph Planner Strategies - Product Requirements

> The current heuristic and model-driven planners only emit bounded linear
> traces. Sift still lacks planner policies that can fork, prioritize, merge,
> and prune graph-shaped autonomous search episodes.

## Problem Statement

Sift has heuristic and model-driven linear planners, but it lacks policies that
can fork, select, merge, and prune graph-shaped autonomous search episodes
under one contract.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Heuristic graph baseline | A deterministic graph planner can fork and prioritize a bounded frontier without model dependence | Graph search runs end to end on a purely local heuristic baseline |
| GOAL-02 | Swappable graph strategies | Heuristic and model-driven graph planning share one explicit strategy surface | Runtime strategy selection works without a second planner contract |
| GOAL-03 | Bounded branching policy | Graph planners remain bounded, replayable, and explicit about fork or prune choices | Planner traces are deterministic enough to test and compare |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Agent Workflow Builder | Developer embedding Sift into a local coding agent. | Needs planner-driven branching search without hand-authoring graph episodes. |
| Power User | User exploring multi-hop local retrieval tasks. | Needs a stronger planner than linear autonomy while staying local-first. |

## Scope

### In Scope

- [SCOPE-01] A heuristic graph planner that can fork and prioritize a bounded frontier.
- [SCOPE-02] A model-driven graph planner strategy under the same graph planner contract.
- [SCOPE-03] Explicit planner strategy selection, graph decisions, and bounded stop semantics.

### Out of Scope

- [SCOPE-04] Hosted or remote planning orchestration.
- [SCOPE-05] General-purpose answer synthesis.
- [SCOPE-06] Parallel branch execution across distributed workers.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must generate graph planner decisions from a root task, branch-local evidence, and current frontier state without requiring caller-authored graph traces. | GOAL-01 | must | This is the core missing planner capability for graph search. |
| FR-02 | The system must ship a heuristic graph planner baseline that can fork, select, and terminate bounded graph episodes without model dependency. | GOAL-01 | must | The mission should have a local deterministic baseline. |
| FR-03 | The system must support explicit graph planner strategy selection so a model-driven graph planner can reuse the same runtime and contract surface. | GOAL-02 | must | Model-driven graph planning should be additive rather than parallel. |
| FR-04 | Graph planner traces must remain bounded and explicit about fork, merge, prune, continue, and terminate decisions. | GOAL-03 | must | Branching policy has to remain replayable and inspectable. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The heuristic graph planner must remain deterministic enough to replay the same graph decisions for the same root task and branch-local evidence. | GOAL-01, GOAL-03 | must | Determinism is critical for testing and regression review. |
| NFR-02 | Model-driven graph planning must remain local-first and bounded by the same explicit graph contract as the heuristic baseline. | GOAL-02, GOAL-03 | must | Strategy selection cannot weaken the product thesis or the runtime contract. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Heuristic baseline | End-to-end local corpus proof and deterministic tests | Story-level verification artifacts linked during execution |
| Strategy selection | Unit or integration tests across heuristic and model-driven graph strategies | Story-level verification artifacts linked during execution |
| Bounded graph behavior | Planner trace review and stop-condition tests | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A useful first graph planner can remain bounded and local rather than needing parallel or remote orchestration. | The mission may widen into infrastructure work. | Validate in evaluation and heuristic baseline voyages. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much branch exploration should the heuristic baseline permit before graph search stops paying for itself? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Sift can autonomously generate and execute bounded graph-shaped search episodes from a root task on local corpora.
- [ ] Heuristic and model-driven graph planner strategies share one explicit contract and runtime.
- [ ] Graph planner traces remain bounded and replayable.
<!-- END SUCCESS_CRITERIA -->
