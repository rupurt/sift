# Plan Sector Maps Frontier Ledgers And Breadcrumb Journals - Software Design Description

> Define a sector-aware restart and frontier-search architecture that can validate unchanged sectors cheaply, resume interrupted indexing, and search partial coverage without waiting for a prior sealed global index.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage defines a restart-aware search architecture that separates cache validity, resumable indexing progress, and partial-search semantics into explicit persisted components:

- `SectorMap` tracks deterministic corpus sectors and their reusable shard outputs.
- `BreadcrumbJournal` records in-flight indexing state so interrupted work can resume.
- `FrontierLedger` provides the minimal statistics and coverage contract needed to search ready sectors before a fully sealed index exists.

The first execution slice should integrate these concepts into direct search so warm restarts can search immediately from prior ready sectors while same-process validation and convergence continue during the active run.

## Context & Boundaries

This voyage is about architecture and decomposition, not full implementation. It stays inside Sift's existing local cache, artifact, and search runtime boundaries.

```
┌──────────────────────────────────────────────────────┐
│                    This Voyage                       │
│                                                      │
│  ┌─────────────┐  ┌────────────────┐  ┌───────────┐ │
│  │ SectorMap   │  │ Breadcrumbs    │  │ Frontier  │ │
│  │ validity    │  │ resumability   │  │ scoring   │ │
│  └─────────────┘  └────────────────┘  └───────────┘ │
└──────────────────────────────────────────────────────┘
          ↑                    ↑                ↑
      [cache]            [search runtime]   [CLI/library]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing artifact/blob cache | Internal | Reuse extracted artifacts and store sector-linked shard outputs | current `sift` cache layout |
| Existing BM25/index runtime | Internal | Provide direct-search retrieval substrate that sector shards will feed | current search runtime |
| Existing progress/telemetry surfaces | Internal | Surface coverage state, reuse, and resume semantics | current CLI/library progress contracts |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Cache validity unit | Sector-level validity proofs, not whole-corpus startup rescans | Unchanged sectors should be reusable without proving the entire corpus from scratch. |
| Partial-search support | First-class `frontier`, `converging`, and `sealed` modes | Frontier hunting must be useful before the corpus is fully sealed. |
| Resume model | Persist breadcrumbs separately from sector validity | Cache proofs and in-flight progress serve different purposes and need different lifecycles. |
| Initial rollout | Direct search first, autonomous/runtime later | Delivers value sooner and constrains the first implementation slice. |
| Existing cache substrate | Extend the current manifest/blob/BM25 cache stack instead of inventing a second file-state tracker | Avoids two competing authorities for file validity and keeps the rollout incremental. |
| Convergence model | Same-process continued validation and rebuild work during the active run, not a background daemon | Preserves the single-binary local-first contract while still allowing frontier or converging search. |

## Architecture

The architecture adds three persisted control-plane structures on top of the existing artifact and BM25 caches:

1. `SectorMap`
   - deterministic sector ids
   - sector membership summary
   - sector validity proof
   - linked shard artifact references
2. `BreadcrumbJournal`
   - active indexing session state
   - completed sectors
   - dirty sectors
   - resumable cursor/progress metadata
3. `FrontierLedger`
   - rolling statistics or merge metadata required to search ready sectors
   - explicit coverage state for frontier versus converged versus sealed search

Search startup should load these persisted structures first, mount ready sectors immediately, and then decide whether same-process validation or rebuild work is required.

The intended rollout order is:

1. sector validity substrate for direct-search startup reuse
2. breadcrumb persistence and restart resume
3. frontier/converging coverage-state signaling and partial-search semantics
4. autonomous and broader library/runtime adoption

## Components

### SectorMap Store

- Purpose: prove which corpus regions are still reusable.
- Interface: load/save/update sector records keyed by deterministic sector id.
- Behavior: validate cheaply from metadata first and only escalate when proofs are stale or ambiguous.
- Integration note: sector proofs should reference the existing manifest/file-heuristic and blob/BM25 artifacts instead of duplicating per-file truth in a parallel index.

### BreadcrumbJournal

- Purpose: resume interrupted indexing instead of starting from file 1.
- Interface: append/update active run state and completed sector checkpoints.
- Behavior: on restart, rehydrate resumable work and allow direct search to mount completed sectors immediately.

### FrontierLedger

- Purpose: support useful search over partial coverage.
- Interface: load rolling stats/merge metadata plus coverage state.
- Behavior: allow ranking over ready sectors before a globally sealed index exists, while making coverage explicit.

### Search Coverage Mode

- Purpose: expose whether a search is `frontier`, `converging`, or `sealed`.
- Interface: progress and response metadata.
- Behavior: inform callers whether ranking is partial, converging, or fully sealed.

## Interfaces

Planned interface seams:

- persisted cache records for `SectorMap`, `BreadcrumbJournal`, and `FrontierLedger`
- startup search path that loads ready sectors before validation completes
- progress/telemetry fields that report:
  - coverage mode
  - reused sectors
  - dirty sectors
  - resumed breadcrumbs
  - converging/sealed transitions

## Data Flow

1. Startup loads `SectorMap`, `BreadcrumbJournal`, and `FrontierLedger`.
2. Search mounts any ready sectors and can begin in `frontier` or `converging` mode immediately.
3. Same-process continued validation checks sector proofs and marks sectors clean or dirty while the active run continues from ready sectors.
4. Dirty sectors rebuild their local artifacts/shards and update the ledger.
5. Coverage moves toward `sealed` as all required sectors converge.
6. Breadcrumbs are updated continuously so restart can resume without replaying the entire corpus walk.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Sector proof missing or invalid | Cache read or proof validation fails | Mark sector dirty and exclude it from sealed reuse claims | Rebuild only the affected sector |
| Breadcrumb journal is incomplete or corrupt | Journal load/parse failure | Drop resumability for that run but keep valid sector reuse | Start a fresh indexing run from the last trusted sector state |
| Frontier ledger is unavailable | Ledger load failure | Fall back to sector-local ranking/merge strategy with explicit frontier coverage | Rebuild ledger incrementally as sectors complete |
| Coverage cannot be stated honestly | Inconsistent ready/dirty/sealed state | Downgrade reported mode to the safest truthful state | Recompute state from persisted sector and breadcrumb records |
