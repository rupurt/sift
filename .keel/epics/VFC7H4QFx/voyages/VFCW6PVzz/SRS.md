# Ship Heuristic Planner Baseline - SRS

## Summary

Epic: VFC7H4QFx
Goal: Generate bounded autonomous turns from a root task using a deterministic heuristic planner over retained evidence and the existing hybrid substrate.

## Scope

### In Scope

- [SCOPE-01] A deterministic heuristic planner that can emit initial and follow-up search decisions from a root task.
- [SCOPE-02] Heuristic query refinement over retained evidence, prior planner decisions, and local context.
- [SCOPE-03] Explicit heuristic stop conditions for bounded linear planning without model-backed planning.

### Out of Scope

- [SCOPE-04] Model-driven planning behavior.
- [SCOPE-05] CLI `sift search --agent` support.
- [SCOPE-06] Branching or graph-search execution.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must ship a heuristic planner policy that can generate an initial autonomous search decision from the root task and current local context without caller-supplied turns. | SCOPE-01 | FR-02 | manual: planner policy review |
| SRS-02 | The heuristic planner must derive and deduplicate follow-up search decisions from retained evidence and prior planner output while staying linear-first. | SCOPE-02 | FR-01 | manual: trace review |
| SRS-03 | The heuristic planner must emit explicit stop reasons when the step limit is reached or when retained evidence no longer yields productive follow-up search decisions. | SCOPE-03 | FR-04 | manual: bounded-run proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The heuristic planner must be deterministic enough to replay the same planner decisions and stop outcomes for the same request and retained evidence. | SCOPE-01, SCOPE-03 | NFR-02 | manual: deterministic test review |
| SRS-NFR-02 | The heuristic planner must remain purely local and model-free so the baseline path preserves the zero-daemon local-first contract. | SCOPE-01, SCOPE-03 | NFR-01 | manual: architecture review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
