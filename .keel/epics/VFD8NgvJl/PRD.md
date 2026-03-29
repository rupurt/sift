# Frontier Runtime and Episode Memory - Product Requirements

> Sift can lower a flat linear planner trace into controller turns, but it
> still lacks a bounded frontier runtime with branch-local evidence and
> explicit merge or prune behavior.

## Problem Statement

The autonomous runtime can only lower a flat list of planned turns, so it lacks
frontier execution, branch-local evidence, and merge or prune behavior for
bounded graph search.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Frontier execution | Graph episodes can execute more than one active branch without inventing a second search stack | A bounded frontier runtime runs graph episodes end to end over the shared substrate |
| GOAL-02 | Branch-local evidence | Retained evidence and state can evolve per branch and resume from explicit frontier state | Branch-local carryover and resume behavior are inspectable and testable |
| GOAL-03 | Merge and prune semantics | Branch selection, merge, and prune behavior are explicit runtime operations | Runtime behavior is bounded, replayable, and observable |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Runtime Maintainer | Developer evolving Sift’s autonomous runtime. | Needs a branchable executor that reuses current controller and retrieval behavior. |
| Agent Workflow Builder | Developer embedding graph search into a local coding agent. | Needs graph episodes to execute and resume without hidden state. |

## Scope

### In Scope

- [SCOPE-01] Frontier execution over graph episode state using the existing hybrid retrieval and controller substrate.
- [SCOPE-02] Branch-local retained evidence, branch status progression, and resume semantics.
- [SCOPE-03] Explicit merge, prune, select, and terminate runtime behavior.

### Out of Scope

- [SCOPE-04] Heuristic or model-driven graph planning policy details.
- [SCOPE-05] Distributed or parallel branch execution.
- [SCOPE-06] Cross-session persistent turn stores.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must provide a bounded frontier runtime that can execute graph episodes over multiple active branches through the shared retrieval and controller substrate. | GOAL-01 | must | Graph search is not real until the runtime can execute graph-shaped episodes. |
| FR-02 | The runtime must preserve branch-local retained evidence and explicit branch status across graph steps and resume points. | GOAL-02 | must | Branches need local memory to remain meaningful and replayable. |
| FR-03 | The runtime must support explicit branch selection, merge, prune, and terminate semantics without hidden frontier mutations. | GOAL-03 | must | Graph runtime behavior has to remain inspectable. |
| FR-04 | The runtime must remain additive to the current linear autonomous and planned-controller paths. | GOAL-01, GOAL-02 | must | Existing runtime behavior must stay stable while graph support lands. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Frontier execution must remain bounded by explicit branch and step limits. | GOAL-03 | must | Graph search cannot sprawl into unbounded local exploration. |
| NFR-02 | Graph runtime state transitions must be replayable and observable from stored graph traces. | GOAL-02, GOAL-03 | must | Runtime inspectability is mandatory for agentic behavior in this repo. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Frontier execution | End-to-end runtime proof over local corpora | Story-level verification artifacts linked during execution |
| Branch-local memory | State progression and replay tests | Story-level verification artifacts linked during execution |
| Merge and prune behavior | Trace review and bounded-run tests | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Shared controller semantics are rich enough to host graph execution without a forked runtime. | The mission may need a deeper runtime split than planned. | Validate in the first runtime voyage. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should merge semantics collapse retained evidence only, or also collapse branch-local trace state? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] A bounded frontier runtime can execute graph episodes through the shared local retrieval substrate.
- [ ] Branch-local retained evidence and explicit branch status can be resumed from stored state.
- [ ] Merge and prune behavior are explicit runtime operations rather than hidden side effects.
<!-- END SUCCESS_CRITERIA -->
