# Refresh Foundational Structural Retrieval Docs - SRS

## Summary

Epic: VG8oL9Xok
Goal: Update the foundational documents so they accurately explain structural fuzzy retrieval, strategy selection, and downstream direct-search adoption.

## Scope

### In Scope

- [SCOPE-01] Update the foundational docs that define the shipped retrieval substrate so they explicitly describe path fuzzy retrieval, segment fuzzy retrieval, and structural reranking.
- [SCOPE-02] Add strategy-selection guidance that explains the role of `path-hybrid`, the richer page-index family, and lower-level retriever overrides.
- [SCOPE-03] Clarify how embedders such as `paddles` should consume the richer direct-search surface without changing planner ownership.

### Out of Scope

- [SCOPE-04] Runtime or algorithm changes to the retrieval pipeline.
- [SCOPE-05] Downstream code changes in `paddles`.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The foundational docs SHALL describe the current structural retrieval stack in terms that match the shipped code and public surface. | SCOPE-01 | FR-01 | manual |
| SRS-02 | The docs SHALL explain strategy and preset selection, including when `path-hybrid` is useful and what the page-index family adds. | SCOPE-02 | FR-02 | manual |
| SRS-03 | The docs SHALL explain the downstream direct-search adoption seam for `paddles`-style embedders without implying that `sift` owns planner behavior. | SCOPE-03 | FR-03 | manual |
| SRS-04 | The repo-wide foundational set SHALL remain internally consistent after the update, including process-level or release-facing references that mention retrieval behavior. | SCOPE-01 | FR-01 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The documentation slice SHALL stay consistent with the current runtime and public crate-root surface. | SCOPE-01 | NFR-01 | manual |
| SRS-NFR-02 | The added guidance SHALL optimize for operator usefulness rather than exhaustive changelog detail. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-02 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
