# Build Frontier Runtime and Branch-Local Evidence - SRS

## Summary

Epic: VFD8NgvJl
Goal: Execute bounded graph episodes over a frontier with branch-local retained evidence and continuation state.

## Scope

### In Scope

- [SCOPE-01] A graph runtime path that can execute more than one active branch over a bounded frontier.
- [SCOPE-02] Branch-local retained evidence carryover, branch status updates, and explicit resume state.
- [SCOPE-03] Additive graph runtime behavior over the shared retrieval and controller substrate.

### Out of Scope

- [SCOPE-04] Merge and prune behavior beyond basic branch-local execution.
- [SCOPE-05] Graph planner policy logic.
- [SCOPE-06] CLI-specific work.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must execute graph episodes over a bounded frontier while reusing the shared retrieval and controller substrate for branch retrieval turns. | SCOPE-01 | FR-01 | manual: runtime proof |
| SRS-02 | The runtime must preserve branch-local retained evidence and branch status between graph steps and resume points. | SCOPE-02 | FR-02 | manual: state progression review |
| SRS-03 | The graph runtime must remain additive to the current linear autonomous path when graph mode is not selected. | SCOPE-03 | FR-04 | manual: regression review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Frontier execution must remain bounded by explicit episode limits and branch limits. | SCOPE-01, SCOPE-02 | NFR-01 | manual: bounded-run proof |
| SRS-NFR-02 | Runtime state progression must remain replayable from stored graph traces and branch-local evidence state. | SCOPE-02 | NFR-02 | manual: replay review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
