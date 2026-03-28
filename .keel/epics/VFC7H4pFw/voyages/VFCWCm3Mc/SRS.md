# Promote Autonomous Library Entry Point - SRS

## Summary

Epic: VFC7H4pFw
Goal: Expose a supported built-in autonomous library entry point that reuses the shared planner runtime without requiring custom planner injection.

## Scope

### In Scope

- [SCOPE-01] A supported crate-root autonomous entry point that invokes built-in planner strategies.
- [SCOPE-02] Public documentation and tests for autonomous planner configuration, strategy selection, and trace usage.
- [SCOPE-03] Strategy-aware autonomous response contracts exposed through the supported library surface.

### Out of Scope

- [SCOPE-04] CLI-specific agent flag plumbing.
- [SCOPE-05] Hosted orchestration or interactive agent shells.
- [SCOPE-06] Branching search APIs.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must expose a supported autonomous library entry point that invokes built-in planner strategies without requiring custom planner injection from embedders. | SCOPE-01 | FR-02 | manual: library proof |
| SRS-02 | The repository documentation must explain how to use autonomous planner strategies, traces, and configuration through the supported library surface. | SCOPE-02 | FR-02 | manual: documentation review |
| SRS-03 | The supported library surface must return strategy-aware autonomous response contracts rather than hiding planner traces behind internal APIs. | SCOPE-01, SCOPE-03 | FR-04 | manual: API review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The supported library surface must reuse the same runtime contract that the future CLI agent path will call. | SCOPE-01, SCOPE-02 | NFR-01 | manual: architecture review |
| SRS-NFR-02 | Existing non-autonomous library modes must remain intact and documented when the autonomous library surface is promoted. | SCOPE-02, SCOPE-03 | NFR-02 | manual: regression review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
