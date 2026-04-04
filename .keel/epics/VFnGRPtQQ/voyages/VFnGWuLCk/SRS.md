# Implement Resumable Sector Rebuild Journals - SRS

## Summary

Epic: VFnGRPtQQ
Goal: Persist sector rebuild breadcrumbs so interrupted indexing resumes from the active sector and mounted clean sectors remain reusable across fresh processes.

## Scope

### In Scope

- [SCOPE-01] Persist breadcrumb journals for sector rebuild runs, including active sector, completed sectors, dirty sectors, and resume cursors.
- [SCOPE-02] Checkpoint rebuild progress during sector processing and persist resumable state under the existing cache root.
- [SCOPE-03] Resume interrupted dirty-sector rebuilds on startup while preserving clean-sector mount behavior.
- [SCOPE-04] Recover safely from stale or corrupt breadcrumb state without invalidating independently reusable sectors.

### Out of Scope

- [SCOPE-05] Frontier/converging/sealed coverage semantics.
- [SCOPE-06] Controller or autonomous runtime adoption.
- [SCOPE-07] Replacing sector validity records introduced in the direct-search voyage.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Persist `BreadcrumbJournal` records with run identity, completed sectors, dirty sectors, active sector state, and resume cursor metadata. | SCOPE-01 | FR-02 | test |
| SRS-02 | Dirty-sector rebuild work checkpoints breadcrumb progress during sector processing so interrupted runs have resumable state. | SCOPE-02 | FR-02 | test |
| SRS-03 | Startup resumes interrupted dirty-sector rebuilds from breadcrumb state while preserving immediate mount of independently clean sectors. | SCOPE-03 | FR-02 | test |
| SRS-04 | Stale or corrupt breadcrumb state is discarded safely without invalidating clean-sector reuse claims backed by sector validity proofs. | SCOPE-04 | FR-02 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Breadcrumb resume and recovery state is observable through runtime telemetry or progress counters for operators and embedders. | SCOPE-03, SCOPE-04 | NFR-03 | test |
| SRS-NFR-02 | Breadcrumb resumability does not require a daemon or any external coordinator outside the local cache root. | SCOPE-01, SCOPE-03 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
