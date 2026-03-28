# Wire Strategy-Selected Autonomous Runtime - SRS

## Summary

Epic: VFC7H4QFx
Goal: Route autonomous planner strategy selection through a built-in runtime that remains bounded, linear, and additive to the current single-turn and controller paths.

## Scope

### In Scope

- [SCOPE-01] A built-in autonomous runtime that can execute planner-generated episodes through the existing controller/search substrate.
- [SCOPE-02] Retained-evidence carryover and planner-state progression across autonomous steps.
- [SCOPE-03] Runtime dispatch that stays additive to current single-turn and planned-controller paths.

### Out of Scope

- [SCOPE-04] Model-driven planner implementation details.
- [SCOPE-05] CLI `sift search --agent` plumbing.
- [SCOPE-06] Branching search execution.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must provide a built-in autonomous runtime path that can execute planner-generated search decisions from a root task without requiring an external custom planner implementation. | SCOPE-01 | FR-01 | manual: end-to-end runtime proof |
| SRS-02 | The runtime must lower planner-generated search decisions into the shared controller/search substrate while carrying retained evidence and planner state across steps. | SCOPE-01, SCOPE-02 | FR-05 | manual: state progression review |
| SRS-03 | The runtime must preserve additive behavior so autonomous planning can resume from explicit planner state without replacing the existing single-turn or planned-controller flows. | SCOPE-02, SCOPE-03 | FR-04 | manual: regression review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Introducing the built-in autonomous runtime must not regress `search_turn` or `search_controller` when autonomous planning is not selected. | SCOPE-03 | NFR-02 | manual: regression proof |
| SRS-NFR-02 | The runtime must reuse shared controller semantics rather than creating a second retained-evidence or multi-turn execution stack. | SCOPE-01, SCOPE-02 | NFR-02 | manual: architecture review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
