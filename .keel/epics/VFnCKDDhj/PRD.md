# Design Sector Maps Breadcrumb Journals And Frontier Search Modes - Product Requirements

## Problem Statement

Sift currently rescans the entire corpus on fresh process startup before it can prove cached BM25 shards are reusable, which makes restart-time frontier hunting too slow and prevents partial sector coverage from being searchable on its own.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Restart-time search reuses previously indexed sectors instead of blocking on a whole-corpus rescan. | A warm restart can begin searching from persisted sector state before full corpus validation completes. | The first useful search result path no longer depends on a full workspace walk. |
| GOAL-02 | Frontier hunting remains useful before the corpus reaches a fully sealed index state. | Partial sector coverage is searchable with explicit coverage semantics. | Frontier, converging, and sealed modes are defined and testable. |
| GOAL-03 | Operators can understand whether search is using sealed, converging, or frontier coverage and whether progress is resumable. | Progress/state reporting distinguishes sector reuse, dirty-sector rebuilds, and breadcrumb resume. | State surfaces are explicit in CLI/library progress contracts. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| CLI Hunter | A developer running `sift search` interactively on large local corpora. | Fast warm restarts and truthful visibility into partial versus sealed coverage. |
| Embedded Integrator | A downstream tool such as `paddles` that calls Sift through the crate-root facade. | A stable restart path that can return useful results without waiting for whole-corpus rescans. |

## Scope

### In Scope

- [SCOPE-01] A persisted `SectorMap` model that partitions the corpus into deterministic reusable sectors with sector-level validity proofs.
- [SCOPE-02] A `BreadcrumbJournal` that records in-progress indexing state and allows interrupted work to resume on the next process.
- [SCOPE-03] A `FrontierLedger` or equivalent partial-coverage scoring model that supports useful search before a fully sealed global index exists.
- [SCOPE-04] Search-mode semantics for `frontier`, `converging`, and `sealed` coverage, including operator-visible signaling.
- [SCOPE-05] Planning the first execution slices required to integrate sector-aware restart reuse into direct search first, then broader autonomous and library search paths.

### Out of Scope

- [SCOPE-06] A long-lived daemon or any external database/service requirement.
- [SCOPE-07] Perfect global ranking parity between frontier mode and fully sealed mode on the first architecture slice.
- [SCOPE-08] User-interface work outside the existing `sift` CLI/library progress and telemetry surfaces.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Define a persisted sector model that lets Sift prove unchanged corpus regions are reusable without reindexing every file on restart. | GOAL-01 | must | Sector-level validity is the foundation for fast warm restarts. |
| FR-02 | Define resumable breadcrumb persistence so interrupted indexing can continue from prior progress rather than restarting from file 1. | GOAL-01, GOAL-03 | must | Resume semantics prevent repeated startup work and make progress truthful. |
| FR-03 | Define a frontier-search mode that can search over ready sectors before a globally sealed index exists. | GOAL-02 | must | Frontier hunting must stay useful before convergence. |
| FR-04 | Define how frontier and converging search compute or approximate ranking statistics without requiring a prior complete global snapshot. | GOAL-02 | must | The architecture must avoid blocking on an initial full seal. |
| FR-05 | Define explicit operator-facing coverage states and progress signals for frontier, converging, and sealed search. | GOAL-03 | should | Callers need to understand what kind of search result they are seeing. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Preserve Sift's single-binary, local-first, zero-daemon contract while introducing sector-aware restart reuse. | GOAL-01, GOAL-02 | must | The optimization cannot require a new operational model. |
| NFR-02 | Maintain clear observability for cache reuse, dirty-sector rebuilds, breadcrumb resume, and coverage state transitions. | GOAL-03 | must | Operators need explicit signals to trust the new restart path. |
| NFR-03 | Keep the design incremental so direct search can benefit first without blocking later autonomous/runtime adoption, and layer the new control plane on the existing manifest/blob/BM25 cache substrate rather than inventing a parallel truth source. | GOAL-01, GOAL-02 | should | The first slice should deliver value without a whole-runtime rewrite or duplicate file-state bookkeeping. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Restart reuse | Architecture review plus story-level tests and CLI proofs | Sector reuse, breadcrumb resume, and warm-restart evidence linked from stories |
| Frontier semantics | Architecture review plus targeted retrieval tests | Coverage-state and partial-search evidence linked from stories |
| Operator visibility | CLI/library progress contract checks | Progress samples and API proof artifacts linked from stories |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Sector-level reuse can be defined deterministically enough to avoid whole-corpus restart scans in the common case. | If false, restart latency remains dominated by corpus validation. | Validate during the architecture voyage. |
| Frontier search can tolerate approximate or rolling corpus statistics as long as coverage is explicit. | If false, useful partial search may require a different retrieval merge strategy. | Validate during the architecture voyage. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which frontier-ranking strategy best balances usefulness and truthfulness before a sealed index exists? | Epic owner | Open |
| How should sector boundaries be chosen so they are stable, cheap to validate, and not too coarse for incremental rebuilds? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The architecture defines how warm restart search begins from persisted sector state instead of waiting for a whole-corpus rescan.
- [ ] The architecture defines searchable frontier/converging/sealed coverage modes without depending on a prior complete global index.
- [ ] The epic is decomposed into execution slices that can ship sector reuse, breadcrumb resume, and coverage signaling incrementally.
<!-- END SUCCESS_CRITERIA -->
