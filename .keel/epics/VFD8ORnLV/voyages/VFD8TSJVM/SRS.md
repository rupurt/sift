# Ship Heuristic Graph Planner Baseline - SRS

## Summary

Epic: VFD8ORnLV
Goal: Introduce a deterministic graph planner that can fork and prioritize a bounded frontier without model dependency.

## Scope

### In Scope

- [SCOPE-01] A deterministic heuristic graph planner that can emit fork, select, continue, and terminate decisions from a root task and branch-local evidence.
- [SCOPE-02] Frontier prioritization and bounded branch expansion without model-backed planning.
- [SCOPE-03] Explicit heuristic stop behavior for exhausted or unproductive graph frontiers.

### Out of Scope

- [SCOPE-04] Model-driven graph planning.
- [SCOPE-05] Parallel or distributed branch execution.
- [SCOPE-06] CLI-specific graph controls.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must ship a heuristic graph planner that can generate graph decisions from a root task, active frontier, and branch-local evidence without caller-authored graph traces. | SCOPE-01 | FR-01 | manual: planner proof |
| SRS-02 | The heuristic planner must fork and prioritize a bounded frontier deterministically enough to replay the same branch decisions for the same input state. | SCOPE-02 | FR-02 | manual: deterministic trace review |
| SRS-03 | The heuristic planner must emit explicit stop reasons when graph exploration is exhausted, unproductive, or bounded by configured limits. | SCOPE-03 | FR-04 | manual: bounded-run proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The heuristic graph planner must remain purely local and model-free so the baseline path preserves the zero-daemon local-first contract. | SCOPE-01, SCOPE-03 | NFR-02 | manual: architecture review |
| SRS-NFR-02 | Heuristic graph traces must remain bounded and deterministic enough for regression review. | SCOPE-02, SCOPE-03 | NFR-01 | manual: deterministic test review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
