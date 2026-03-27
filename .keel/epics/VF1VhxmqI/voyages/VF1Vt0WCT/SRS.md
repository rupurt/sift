# Define Turn Model and Emission Contract - SRS

## Summary

Epic: VF1VhxmqI
Goal: Make agentic behavior explicit as stable turn-oriented data and emission contracts without breaking the current single-turn hybrid path.

## Scope

### In Scope

- [SCOPE-01] Add turn-native request, response, controller-state, and trace records.
- [SCOPE-02] Define explicit emission modes that separate view/protocol/latent outputs.
- [SCOPE-03] Expose supported agentic-facing contracts at a stable API boundary.

### Out of Scope

- [SCOPE-04] Implement the full multi-turn controller loop.
- [SCOPE-05] Add benchmark fixtures or comparative reporting.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must introduce turn-oriented search request and response types that can represent multi-turn controller state explicitly. | SCOPE-01 | FR-01 | manual: API review |
| SRS-02 | The system must define trace records that can capture turn progression and controller decisions without relying on implicit runtime state. | SCOPE-01 | FR-01 | manual: API review |
| SRS-03 | The system must define explicit emission modes for view, protocol, and latent-oriented outputs. | SCOPE-02 | FR-02 | manual: type and API review |
| SRS-04 | The system must expose the new contracts through a supported public API boundary rather than only through `sift::internal`. | SCOPE-03 | FR-03 | manual: embedder proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Existing single-turn hybrid search behavior must remain available while the new contracts are introduced. | SCOPE-01 | NFR-01 | manual: code review |
| SRS-NFR-02 | Turn and emission contracts must remain serializable or otherwise inspectable for trace and test usage. | SCOPE-02 | NFR-02 | manual: code review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
