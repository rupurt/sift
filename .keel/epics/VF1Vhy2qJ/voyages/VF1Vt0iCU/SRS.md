# Build Local Multi-Turn Loop Execution - SRS

## Summary

Epic: VF1Vhy2qJ
Goal: Implement a bounded local controller that decomposes queries, reuses the existing hybrid substrate across turns, manages context, and terminates deterministically.

## Scope

### In Scope

- [SCOPE-01] Add deterministic multi-turn controller state and stop conditions.
- [SCOPE-02] Reuse the existing hybrid retrieval substrate inside the turn loop.
- [SCOPE-03] Add bounded context-retention or pruning mechanics.
- [SCOPE-04] Expose controller invocation through supported CLI or library entry points.

### Out of Scope

- [SCOPE-05] Train or ship a dedicated search-agent model.
- [SCOPE-06] Add hosted orchestration or background services.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must execute a deterministic multi-turn controller loop over the current local hybrid retrieval substrate. | SCOPE-01 | FR-01 | manual: end-to-end local corpus proof |
| SRS-02 | The controller must derive turn actions from explicit controller state and plan data rather than hidden runtime overrides. | SCOPE-02 | FR-02 | manual: architecture review |
| SRS-03 | The controller must manage bounded context by retaining, discarding, or otherwise curating retrieved evidence across turns. | SCOPE-03 | FR-03 | manual: end-to-end trace proof |
| SRS-04 | The system must expose a supported invocation path for multi-turn search in the CLI, library, or both. | SCOPE-04 | FR-04 | manual: CLI or embedder proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Multi-turn execution must preserve the zero-daemon, local-first contract. | SCOPE-01 | NFR-01 | manual: architecture review |
| SRS-NFR-02 | The single-turn hybrid path must not be regressed when the controller is not selected. | SCOPE-02 | NFR-02 | manual: regression review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
