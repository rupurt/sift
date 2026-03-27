# Turn-Native Search Contract and Emissions - Product Requirements

> The current hybrid runtime can retrieve and rerank well, but it still cannot express agentic search behavior as stable turn-oriented data or emit non-CLI search artifacts cleanly.

## Problem Statement

Sift has agentic direction and model hooks, but it still lacks a first-class turn-oriented domain contract, explicit emission modes, and a stable public surface for agentic search behavior.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Turn-native contracts | Agentic search state is modeled as explicit domain data | Turn-oriented request/response and trace types exist in code |
| GOAL-02 | Explicit emissions | Search can emit more than CLI-shaped file hits | Visual, protocol, and latent emission modes are defined and exercised |
| GOAL-03 | Stable public surface | Embedders can invoke the new contracts without reaching into unstable internals | Crate-root or clearly supported facade exposes the required agentic entry points |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Embedder | Rust developer embedding sift into a larger local agent workflow. | Needs turn-oriented contracts and emissions that are stable enough to build on. |
| CLI Power User | User experimenting with complex local retrieval tasks. | Needs search outputs that can evolve beyond a single file-hit view. |

## Scope

### In Scope

- [SCOPE-01] Add turn-native request, response, and trace-facing domain types.
- [SCOPE-02] Define emission modes for visual, protocol, and latent outputs.
- [SCOPE-03] Expose a supported public API boundary for these contracts.

### Out of Scope

- [SCOPE-04] Full multi-turn controller logic and context-pruning heuristics.
- [SCOPE-05] Benchmarking or evaluation harnesses beyond contract-level verification.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must model agentic search turns, controller state, and trace records as explicit data types rather than implicit runtime behavior. | GOAL-01 | must | Agentic execution cannot be inspectable if the state is not first-class. |
| FR-02 | The system must support explicit emission modes so callers can request view/protocol/latent-oriented outputs from the same retrieval substrate. | GOAL-02 | must | Agentic search needs structured outputs, not only file-rendered results. |
| FR-03 | The supported public API must expose the new turn and emission contracts without forcing embedders into `sift::internal`. | GOAL-03 | must | The pivot needs a stable interface, not only internal scaffolding. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The new contracts must preserve backward-compatible single-turn hybrid search behavior for current CLI and library users. | GOAL-01 | must | The pivot cannot regress the shipped retrieval path. |
| NFR-02 | Contract and emission types must remain serializable and inspectable for traces, tests, and downstream tooling. | GOAL-02 | must | Agentic workflows need replayable artifacts. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Domain contract | Unit tests and API review | Story evidence proving explicit turn and emission types |
| Public API | Embedding proof and compile-time usage | Story evidence showing supported facade access |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Existing hybrid retrieval types can be extended without forcing an immediate full controller rewrite. | The epic may need deeper invasive refactors. | Validate during voyage design. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much of the current `SearchResponse` shape can remain stable while introducing richer emissions? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Turn-oriented contracts exist as explicit code-level types.
- [ ] Emission modes beyond the current CLI-shaped view are designed and implemented.
- [ ] Embedders can access supported agentic-facing contracts without depending on unstable internals.
<!-- END SUCCESS_CRITERIA -->
