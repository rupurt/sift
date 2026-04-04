# Plan Sector Maps Frontier Ledgers And Breadcrumb Journals - SRS

## Summary

Epic: VFnCKDDhj
Goal: Define a sector-aware restart and frontier-search architecture that can validate unchanged sectors cheaply, resume interrupted indexing, and search partial coverage without waiting for a prior sealed global index.

## Scope

### In Scope

- [SCOPE-01] A canonical `SectorMap` contract that partitions a corpus into deterministic reusable sectors.
- [SCOPE-02] Sector-level validity proofs using sector hashes derived from cheap metadata first and stronger proofs when needed.
- [SCOPE-03] A `BreadcrumbJournal` contract for interrupted indexing and restart resume.
- [SCOPE-04] A `FrontierLedger` or equivalent contract for searching partial sector coverage before a fully sealed index exists.
- [SCOPE-05] Explicit coverage-state semantics, progress reporting, and first-slice decomposition for direct search integration.

### Out of Scope

- [SCOPE-06] Implementing the full runtime end to end in this voyage.
- [SCOPE-07] Requiring a daemon, external database, or background service.
- [SCOPE-08] Perfect frontier-mode ranking parity with a fully sealed index and non-search UI work outside existing progress/reporting surfaces.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The design shall define a persisted `SectorMap` schema that records sector identity, membership summary, validity proof, and linked search shards. | SCOPE-01 | FR-01 | review |
| SRS-02 | The design shall define how unchanged sectors are validated cheaply on restart without reparsing the full corpus in the common case. | SCOPE-02 | FR-01 | review |
| SRS-03 | The design shall define a persisted `BreadcrumbJournal` that records completed sectors, active sector work, and resumable indexing state across process restarts. | SCOPE-03 | FR-02 | review |
| SRS-04 | The design shall define a `frontier` search mode that can search ready sectors before a fully sealed corpus index exists. | SCOPE-04 | FR-03 | review |
| SRS-05 | The design shall define how frontier and converging search obtain usable scoring statistics without waiting for a prior complete global snapshot. | SCOPE-04 | FR-04 | review |
| SRS-06 | The design shall define explicit coverage-state signaling for frontier, converging, and sealed search through progress and API surfaces. | SCOPE-05 | FR-05 | review |
| SRS-07 | The voyage shall decompose implementation into ordered execution slices: sector validity substrate in direct search startup, breadcrumb resume, frontier/converging coverage signaling, and later autonomous/library adoption. | SCOPE-05 | FR-01 | review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The design shall preserve Sift's single-binary local-first contract and avoid introducing a required daemon or external service. | SCOPE-01, SCOPE-04 | NFR-01 | review |
| SRS-NFR-02 | The design shall make restart reuse, breadcrumb resume, and coverage-state transitions observable to operators and embedders. | SCOPE-03, SCOPE-05 | NFR-02 | review |
| SRS-NFR-03 | The design shall allow incremental adoption so direct search can ship first without blocking later autonomous/runtime integration, and it shall extend the existing manifest/blob/BM25 cache substrate rather than defining a second file-state authority. | SCOPE-05 | NFR-03 | review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
