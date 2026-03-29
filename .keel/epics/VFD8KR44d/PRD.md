# Graph Episode Contract and Replay - Product Requirements

> Sift ships linear autonomous planning, but it still lacks a first-class graph
> episode contract for branching decisions, frontier state, and replayable graph
> traces.

## Problem Statement

Sift only has linear planner state and flat traces, so it cannot represent
branching decisions, frontier state, or replayable graph episodes as first-class
data.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Graph episode contract | Branching search can be expressed as supported request, response, state, node, edge, and frontier records | A graph episode DTO family exists without replacing the linear planner contract |
| GOAL-02 | Replayable graph traces | Graph decisions, branch status, and episode completion can be reconstructed from stored traces and validation rules | Invalid branch references are rejected and valid episodes can be replayed deterministically |
| GOAL-03 | Additive compatibility | The graph contract extends rather than breaks the existing autonomous library surface | Current linear planner callers remain supported while graph mode is introduced |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Library Integrator | Developer embedding Sift into an agentic coding workflow. | Needs graph episodes represented as stable data before wiring graph execution or evaluation. |
| Runtime Maintainer | Developer evolving the autonomous runtime. | Needs a graph contract that can host frontier execution and replay without hidden state. |

## Scope

### In Scope

- [SCOPE-01] Graph episode request, response, state, node, edge, branch-status, and frontier records.
- [SCOPE-02] Replayable graph trace data and validation rules for explicit branch transitions.
- [SCOPE-03] Additive extension points that preserve the existing linear autonomous contract while introducing graph mode.

### Out of Scope

- [SCOPE-04] Frontier execution policy and runtime scheduling.
- [SCOPE-05] Heuristic or model-driven graph planning behavior.
- [SCOPE-06] CLI-specific graph controls.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must introduce supported graph episode records that represent graph mode, branch-local state, frontier membership, node identifiers, and branch status explicitly. | GOAL-01 | must | Branching search cannot be embedded safely if graph state remains implicit. |
| FR-02 | The system must represent graph planner decisions and trace steps as replayable data with explicit node and edge references. | GOAL-02 | must | Inspectability is a repository requirement for agentic behavior. |
| FR-03 | The system must validate graph traces so impossible node, edge, or frontier transitions are rejected deterministically. | GOAL-02 | must | Replay needs validation, not just storage. |
| FR-04 | The graph contract must extend the current autonomous surface without breaking the existing linear autonomous request or response path. | GOAL-03 | must | The next mission should build on the current shipped autonomy rather than replacing it. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The graph contract must remain bounded and replayable, with stable node and branch identifiers that later runtime work can execute without reinterpretation. | GOAL-02 | must | The contract should be execution-ready instead of narrative-only. |
| NFR-02 | The current linear autonomous path must remain intact when graph mode is not selected. | GOAL-03 | must | Graph work must stay additive during rollout. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Graph DTO contract | API review plus serialization and unit tests | Story-level verification artifacts linked during execution |
| Trace validation | Deterministic replay and contract tests | Story-level verification artifacts linked during execution |
| Additive compatibility | Regression review against current autonomous APIs | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A graph contract can extend the current autonomous DTO family without forcing a breaking API split. | The runtime mission may require a harder migration than planned. | Validate during voyage-level contract design. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should graph mode introduce a dedicated request type or remain a mode on the current autonomous request surface? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Supported graph episode state, trace, and branch records exist at the intended public layer.
- [ ] Graph traces can be validated and replayed deterministically from explicit node and frontier data.
- [ ] The graph contract is additive to the shipped linear autonomous surface.
<!-- END SUCCESS_CRITERIA -->
