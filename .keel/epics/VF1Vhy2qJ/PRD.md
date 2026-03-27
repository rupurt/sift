# Local Multi-Turn Search Controller - Product Requirements

> Sift already has a strong local hybrid retrieval substrate, but it still executes as a single pass. The missing layer is a bounded controller that can plan, search, retain, prune, and stop over multiple turns.

## Problem Statement

The current runtime is a single-pass pipeline. Sift needs a reusable local controller that decomposes queries, iterates retrieval, manages bounded context, and terminates deterministically over the existing hybrid substrate.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Query decomposition and turn loop | Complex queries can trigger repeated retrieval turns with explicit state transitions | A local multi-turn controller runs end-to-end on a local corpus |
| GOAL-02 | Shared substrate reuse | Multi-turn search reuses the existing hybrid retrieval pipeline instead of forking a separate stack | Controller delegates retrieval to the same retriever/fusion/reranker substrate |
| GOAL-03 | Deterministic bounded behavior | The controller stops predictably and manages context within explicit limits | Termination and bounded-context rules are encoded and tested |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Agent Workflow Builder | Developer wiring sift into local coding or research agents. | Needs a reusable local controller rather than hand-rolled retrieval loops. |
| Power User | CLI or library user exploring multi-hop local search. | Needs better recall on complex tasks without abandoning the local-first model. |

## Scope

### In Scope

- [SCOPE-01] Controller state, turn loop, and deterministic stop conditions.
- [SCOPE-02] Context-retention or pruning mechanics over retrieved evidence.
- [SCOPE-03] Reuse of the existing hybrid retrieval substrate within controller execution.
- [SCOPE-04] CLI and library entry points that can invoke multi-turn search.

### Out of Scope

- [SCOPE-05] Training or shipping a specialized search-agent model.
- [SCOPE-06] Remote orchestration, hosted storage, or service-based execution.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must execute a bounded multi-turn search loop over the existing local hybrid substrate. | GOAL-01 | must | This is the core missing runtime capability. |
| FR-02 | The controller must make turn decisions from explicit state rather than hidden overrides or background mutation. | GOAL-03 | must | Agentic behavior must be inspectable and replayable. |
| FR-03 | The controller must support context management decisions that retain relevant evidence and discard stale or redundant evidence within a bounded budget. | GOAL-01 | must | Long-horizon search is not viable without bounded context. |
| FR-04 | The system must expose the controller through supported CLI or library entry points. | GOAL-02 | must | The runtime is not real until users can invoke it. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Multi-turn execution must preserve the zero-daemon, local-first contract. | GOAL-02 | must | This is a repository constitution-level constraint. |
| NFR-02 | Multi-turn execution must not unjustifiably degrade the current single-turn hybrid champion when the controller is not in use. | GOAL-02 | must | The pivot cannot regress the substrate. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Controller behavior | End-to-end CLI or library proof on local corpora | Story-level verification artifacts |
| Determinism | Unit tests around turn transitions and termination | Story-level verification artifacts |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Local model quality is sufficient for an initial controller to add value without frontier-hosted reasoning. | The first controller may be too weak to justify the complexity. | Validate in the evaluation epic. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which initial decomposition and pruning heuristics are strong enough before learned policies exist? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Sift can run a bounded multi-turn search loop on a local corpus.
- [ ] The controller reuses the existing hybrid retrieval substrate rather than a forked stack.
- [ ] Users can invoke the controller through a supported surface.
<!-- END SUCCESS_CRITERIA -->
