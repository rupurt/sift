# Add Model-Driven Planner Strategy - SRS

## Summary

Epic: VFC7H4QFx
Goal: Introduce a local-first model-driven planner strategy that reuses the shared autonomous planner contract and bounded linear runtime.

## Scope

### In Scope

- [SCOPE-01] A local-first model-driven planner adapter that implements the shared planner contract.
- [SCOPE-02] Strategy selection and profile routing between heuristic and model-driven planning.
- [SCOPE-03] Strategy-aware planner traces and explicit handling for unavailable model-driven configurations.

### Out of Scope

- [SCOPE-04] Hosted or remote planning orchestration.
- [SCOPE-05] CLI flag plumbing.
- [SCOPE-06] Branching search behavior.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must implement a model-driven planner adapter that emits search, continue, and terminate decisions through the same contract used by the heuristic planner. | SCOPE-01 | FR-03 | manual: planner adapter proof |
| SRS-02 | Planner strategy kind and profile must route runtime execution between heuristic and model-driven planning through one selection surface. | SCOPE-02 | FR-03 | manual: strategy selection review |
| SRS-03 | Planner traces and responses must record which planner strategy executed a run so evaluation and replay can compare heuristic and model-driven behavior. | SCOPE-03 | FR-03 | manual: trace review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The model-driven planner path must remain local-first and zero-daemon even when it relies on an in-process generative model. | SCOPE-01, SCOPE-02 | NFR-01 | manual: architecture review |
| SRS-NFR-02 | Model-driven strategy selection must preserve bounded linear semantics and fail explicitly when the requested planner profile is unavailable. | SCOPE-02, SCOPE-03 | NFR-02 | manual: runtime review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
