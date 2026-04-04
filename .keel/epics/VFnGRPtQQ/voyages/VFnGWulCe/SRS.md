# Adopt Sector Reuse Across Runtime Surfaces - SRS

## Summary

Epic: VFnGRPtQQ
Goal: Route controller, autonomous, and library runtime surfaces through the sector-aware preparation path and prove restart reuse end to end.

## Scope

### In Scope

- [SCOPE-01] Route controller and autonomous search startup through the shared sector-aware preparation path instead of separate whole-corpus startup logic.
- [SCOPE-02] Expose the same sector reuse and coverage semantics across direct, controller, autonomous, CLI, and library runtime surfaces.
- [SCOPE-03] Reuse one shared cache root and prepared sector state across fresh processes and across runtime surfaces.
- [SCOPE-04] Prove end-to-end restart reuse and bounded dirty-sector rebuild behavior across runtime surfaces, and document the rollout for operators and embedders.

### Out of Scope

- [SCOPE-05] New retrieval or ranking algorithms beyond adoption of the shared preparation path.
- [SCOPE-06] Provider-specific UI or workflow concerns outside sift runtime surfaces.
- [SCOPE-07] Native 1-bit or alternative runtime execution work.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Controller and autonomous search startup uses the shared sector-aware preparation path instead of duplicate whole-corpus rebuild logic. | SCOPE-01 | FR-04 | test |
| SRS-02 | Controller, autonomous, CLI, and library runtime surfaces expose the same sector reuse and coverage metrics contract as direct search. | SCOPE-02 | FR-04 | test |
| SRS-03 | Fresh-process runs can reuse clean sectors prepared by any supported runtime surface through the shared cache root. | SCOPE-03 | FR-04 | command |
| SRS-04 | End-to-end proofs demonstrate bounded dirty-sector rebuilds and shared cache reuse across runtime surfaces, with operator-facing documentation of the behavior. | SCOPE-04 | FR-04 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Runtime adoption preserves one preparation authority and one cache substrate instead of branching into per-surface startup implementations. | SCOPE-01, SCOPE-03 | NFR-02 | manual |
| SRS-NFR-02 | The rollout preserves sift's local-first, library-friendly behavior and documents the shared cache semantics for downstream embedders. | SCOPE-02, SCOPE-04 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
