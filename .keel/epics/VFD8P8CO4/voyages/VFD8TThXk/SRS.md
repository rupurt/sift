# Promote Graph Search Library and CLI Surface - SRS

## Summary

Epic: VFD8P8CO4
Goal: Expose bounded graph search through the supported library surface and existing sift search --agent CLI.

## Scope

### In Scope

- [SCOPE-01] A supported library-facing graph search surface that builds on the current autonomous contracts and runtime patterns.
- [SCOPE-02] CLI support for graph mode through the existing `sift search --agent` path.
- [SCOPE-03] Documentation and output metadata that make graph mode and graph traces inspectable.

### Out of Scope

- [SCOPE-04] A second standalone autonomous CLI command.
- [SCOPE-05] Interactive graph shells or chat UIs.
- [SCOPE-06] Hosted orchestration features.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must expose a supported library-facing graph search path that allows embedders to select bounded graph search without relying on internal-only seams. | SCOPE-01 | FR-03 | manual: library integration review |
| SRS-02 | The system must expose graph search through the existing `sift search --agent` entry point rather than a second autonomous command. | SCOPE-02 | FR-04 | manual: CLI proof |
| SRS-03 | Library and CLI graph responses must expose enough graph metadata to support inspection and regression review. | SCOPE-03 | FR-03 | manual: output review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Adding graph search surfaces must not regress direct search or the current linear autonomous library and CLI paths. | SCOPE-01, SCOPE-02 | NFR-01 | manual: regression review |
| SRS-NFR-02 | Graph surface output must remain bounded and deterministic enough for regression review and user inspection. | SCOPE-03 | NFR-02 | manual: output stability review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
