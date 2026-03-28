# Planner Contract and Replay - Product Requirements

> Sift can replay explicit planned turns, but it still lacks a first-class planner contract that can represent autonomous intent, planner strategy, planner decisions, and explicit stop semantics as data.

## Problem Statement

Sift can replay explicit planned turns but has no first-class planner contract, planner state, or replayable stop semantics for autonomous planning.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Planner contract | Autonomous planning can be expressed as supported request, response, state, and decision records | A library-facing planner contract exists without relying on ad hoc structs or logs |
| GOAL-02 | Replayable stop semantics | Planner continuation and termination behavior can be reconstructed from stored state and trace records | Stop reasons and planner decisions are explicit and testable |
| GOAL-03 | Strategy extensibility | Heuristic and model-driven planning can share one contract | Planner policy is explicit data rather than an implicit special case |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Library Integrator | Developer embedding Sift into an agentic coding or research workflow. | Needs a stable autonomous-planning contract before wiring planner-driven retrieval. |
| Runtime Maintainer | Developer evolving the search runtime. | Needs planner semantics separated from the current planned-controller loop. |

## Scope

### In Scope

- [SCOPE-01] Planner-facing request, response, state, strategy, decision, and stop-reason records.
- [SCOPE-02] A library-facing autonomous execution seam that can host planner output while reusing the current retrieval/controller substrate.
- [SCOPE-03] Explicit replay and trace semantics for planner decisions and linear continuation/termination behavior.

### Out of Scope

- [SCOPE-04] The full autonomous planner policy that invents production-ready turns end to end.
- [SCOPE-05] CLI `--agent` support.
- [SCOPE-06] Public branching or graph-search APIs.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must introduce supported planner request, response, and state records that represent a root task, retained evidence, planner strategy, current step, and completion state explicitly. | GOAL-01 | must | Autonomous planning cannot be embedded safely if the contract is implicit. |
| FR-02 | The system must record planner decisions and termination reasons as replayable data rather than relying on log output or hidden controller behavior. | GOAL-02 | must | Inspectability is a repository-level requirement for agentic behavior. |
| FR-03 | The system must define a planner-strategy contract that can host both heuristic and model-driven planning without changing the calling surface. | GOAL-03 | must | The mission explicitly wants heuristic and model-driven planning under one extensible seam. |
| FR-04 | The system must expose a library-first autonomous execution seam that composes planner output with the existing retrieval/controller runtime while remaining linear-first. | GOAL-01, GOAL-02 | must | The planner contract is not complete until it can be exercised through a supported runtime seam. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The planner contract must remain linear-first while encoding enough stable identifiers and reason codes to extend toward branching search later without a hard cutover. | GOAL-03 | must | The user wants extensibility to graph search without widening mission scope now. |
| NFR-02 | The current single-turn and planned-controller paths must not regress when the planner contract is introduced. | GOAL-02 | must | The contract layer must be additive until the planner policy is proven. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Planner contract | API review plus serialization/unit tests | Story-level verification artifacts linked during execution |
| Replay semantics | Trace and state inspection from deterministic tests | Story-level verification artifacts linked during execution |
| Runtime seam | End-to-end library proof over a local corpus | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Linear planner state can be structured in a way that future branching search can extend rather than replace. | A later branching mission may require an incompatible public break. | Validate in voyage design and contract review. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should planner state directly embed current controller state or wrap it through a more general episode model? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Supported planner request, response, state, strategy, and decision records exist at the intended public layer.
- [ ] Planner continuation and termination can be reconstructed from explicit state and trace records.
- [ ] A library-facing autonomous execution seam exists without introducing a public graph API.
<!-- END SUCCESS_CRITERIA -->
