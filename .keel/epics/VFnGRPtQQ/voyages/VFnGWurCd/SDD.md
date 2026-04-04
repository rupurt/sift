# Implement Frontier Coverage Search Semantics - Software Design Description

> Support truthful frontier, converging, and sealed search over mounted sectors with explicit coverage signaling and rolling lexical statistics.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds truthful runtime coverage semantics on top of sector reuse and breadcrumb resume. It introduces a frontier ledger derived from the sector map and breadcrumb journal, then threads that state through progress reporting and direct-search responses without reintroducing whole-corpus startup work.

## Context & Boundaries

This slice covers direct-search observability and search-result truthfulness only. It does not yet route controller or autonomous surfaces through the shared preparation path, but it establishes the coverage contract those surfaces will adopt in the next voyage.

```
┌────────────────────────────────────────────────────────────┐
│                 Direct Search Coverage                     │
│                                                            │
│  sector map + breadcrumbs -> frontier ledger -> progress   │
│                         -> coverage-tagged results         │
└────────────────────────────────────────────────────────────┘
             ↑                                   ↑
      sector reuse slice                  direct search callers
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/cache/sector.rs` + `src/cache/store.rs` | internal | Existing sector validity substrate that coverage state must summarize | current repo |
| `src/cache/breadcrumb.rs` | internal | Resume state that tells coverage whether rebuild is still converging | current repo |
| `src/search/domain.rs` + `src/system.rs` | internal | Runtime progress and telemetry contracts that will expose coverage state | current repo |
| `src/facade.rs` + `src/main.rs` | internal | Library and CLI direct-search surfaces that need truthful coverage reporting | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Coverage authority | Derive from sector map plus breadcrumb state | Truthful coverage must reflect the same cache facts that drive reuse and rebuild. |
| Conservative state transitions | `sealed` only after all reachable dirty sectors complete | Prevents false completeness claims during interrupted or partial indexing. |
| Visibility surface | Expose both mode and counts | Operators need a compact status word and enough numbers to understand progress. |

## Architecture

Add a frontier coverage layer that sits between sector preparation and caller-visible progress:

1. sector map and breadcrumb journal feed a runtime `FrontierLedger`
2. the ledger computes coverage mode and rolling counts
3. direct-search progress and response types carry that snapshot outward
4. resume and recovery paths update the ledger conservatively as state changes

## Components

### Frontier Ledger

Purpose: summarize mounted, dirty, rebuilding, completed, skipped, and reused sectors into one runtime coverage snapshot.

Likely files:
- `src/cache/frontier.rs`
- `src/search/corpus.rs`

### Coverage Evaluator

Purpose: turn ledger state into `frontier`, `converging`, or `sealed` at startup and throughout rebuild progress.

Likely files:
- `src/search/domain.rs`
- `src/system.rs`
- `src/facade.rs`

### Direct Search Surface Adapter

Purpose: expose coverage metadata through progress callbacks, snapshots, and search results without changing the direct-search ownership boundary.

Likely files:
- `src/facade.rs`
- `src/main.rs`

## Interfaces

Planned internal and public interfaces:

- frontier ledger load/build helpers derived from sector and breadcrumb state
- progress snapshot extensions carrying coverage mode and sector counters
- direct-search result metadata extensions that report whether results are `frontier`, `converging`, or `sealed`

## Data Flow

1. Direct-search startup loads clean sectors and any breadcrumb resume state.
2. Coverage preparation derives a frontier ledger from mounted, dirty, active, and completed sectors.
3. The ledger computes initial coverage mode before useful results are surfaced.
4. Dirty-sector rebuild and resume events update the ledger as sectors converge.
5. Progress callbacks and direct-search responses emit the current coverage snapshot.
6. Once every reachable dirty sector converges, the mode flips to `sealed`.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Frontier ledger missing or stale | No persisted summary or counters conflict with sector map | Recompute ledger from sector and breadcrumb state | Overwrite stale ledger on the next successful write |
| Breadcrumb indicates active rebuild but sector proofs are complete | Resume metadata conflicts with clean-sector state | Favor the conservative `converging` state until the rebuild path reconciles state | Clear stale active markers once validation completes |
| Coverage surface cannot serialize new metadata | Result/progress conversion fails schema checks | Fall back to emitting a minimal conservative snapshot | Preserve search execution and log the surface mismatch |
