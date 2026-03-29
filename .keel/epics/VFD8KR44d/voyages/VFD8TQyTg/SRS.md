# Validate Replayable Graph Traces - SRS

## Summary

Epic: VFD8KR44d
Goal: Add validation and deterministic replay guarantees for branchable planner traces before policy work begins.

## Scope

### In Scope

- [SCOPE-01] Graph trace validation for node, edge, frontier, and branch references.
- [SCOPE-02] Deterministic replay rules that can reconstruct graph frontier progression and completion state.
- [SCOPE-03] Explicit contract errors for impossible graph transitions.

### Out of Scope

- [SCOPE-04] Heuristic or model-driven graph planning logic.
- [SCOPE-05] Runtime scheduling policy.
- [SCOPE-06] CLI evaluation or surface work.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must validate graph traces so branch, node, edge, and frontier references cannot point at missing or impossible graph state. | SCOPE-01 | FR-03 | manual: validation proof |
| SRS-02 | The system must be able to replay a validated graph trace and reconstruct frontier progression, branch status, and episode completion deterministically. | SCOPE-02 | FR-02 | manual: replay review |
| SRS-03 | Validation failures must surface explicit contract errors rather than silently repairing invalid graph traces. | SCOPE-03 | FR-03 | manual: error review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Replay outcomes must remain deterministic for the same validated graph trace and initial graph state. | SCOPE-02 | NFR-01 | manual: deterministic test review |
| SRS-NFR-02 | Graph trace validation must stay additive to the current linear trace path when graph mode is not selected. | SCOPE-01, SCOPE-03 | NFR-02 | manual: regression review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
