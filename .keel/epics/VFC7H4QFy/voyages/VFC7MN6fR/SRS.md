# Define Planner State and Stop Semantics - SRS

## Summary

Epic: VFC7H4QFy
Goal: Make planner decisions, continuation criteria, and linear stop semantics explicit and replayable while keeping the design extensible to future branching search.

## Scope

### In Scope

- [SCOPE-01] Add planner-facing request, response, state, decision, and stop-reason records.
- [SCOPE-02] Add explicit planner strategy selection that can host heuristic and model-driven policies.
- [SCOPE-03] Define a library-first autonomous execution seam that can compose planner output with the current retrieval/controller runtime while preserving existing single-turn and planned-controller behavior.

### Out of Scope

- [SCOPE-04] Implementing the full autonomous planner policy.
- [SCOPE-05] CLI `sift search --agent` support.
- [SCOPE-06] Branching or graph-search execution.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must introduce supported autonomous-planning request, response, and planner-state records that can represent a root task, planner strategy, current linear step, retained evidence, and completion status explicitly. | SCOPE-01 | FR-01 | manual: API review |
| SRS-02 | The system must introduce explicit planner decision and stop-reason records so continuation and termination can be replayed without relying on log output. | SCOPE-01 | FR-02 | manual: trace review |
| SRS-03 | The system must define a planner-strategy contract that allows heuristic and model-driven policies to share one calling surface. | SCOPE-02 | FR-03 | manual: API review |
| SRS-04 | The system must expose a library-first autonomous execution seam that can host planner-driven search while reusing the current retrieval/controller runtime. | SCOPE-03 | FR-04 | manual: end-to-end library proof |
| SRS-05 | The new planner contract must preserve current single-turn and planned-controller invocation paths when autonomous planning is not selected. | SCOPE-03 | FR-04 | manual: regression review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The planner contract must remain linear-first while carrying enough stable planner identifiers and reason codes to extend toward branching search later without replacing the public contract. | SCOPE-01 | NFR-01 | manual: architecture review |
| SRS-NFR-02 | Introducing the planner contract must not regress current single-turn search or deterministic planned-controller execution. | SCOPE-03 | NFR-02 | manual: regression review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
