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

Proposed persisted record:

| Field | Meaning |
|-------|---------|
| `sector_id` | Deterministic id derived from corpus root + sector partition key |
| `partition_key` | Stable sector boundary descriptor such as directory shard or bounded file bucket |
| `member_summary` | File count, relative-path range, and aggregate byte count for the sector |
| `proof_kind` | `metadata`, `content`, or `dirty` |
| `proof_material` | Cheap manifest-derived heuristics fingerprint and optional escalated content digest |
| `artifact_refs` | Existing blob/manifests references for extracted artifacts that belong to the sector |
| `bm25_shard_ref` | Reference to the sector-local lexical shard used during reuse |
| `last_validated_at` | Timestamp for the last trusted validation pass |
| `state` | `clean`, `dirty`, or `rebuilding` |

Sector partitioning rules:

- deterministic from the corpus root and file membership, never from transient process order
- small enough that a few dirty files do not invalidate the whole corpus
- large enough that a mounted sector can contribute useful lexical statistics on its own

Validation ladder:

1. Load the sector membership summary and compare manifest-derived file heuristics (`inode`, `mtime`, `size`) for member files.
2. If all heuristics still match and referenced artifacts/shards exist, the sector remains `clean` without reparsing file content.
3. If heuristics drift or required artifacts are missing, escalate only that sector to a stronger proof using existing content hashes or shard rebuild.
4. Mark the sector `dirty` when proof escalation fails or when sector membership can no longer be stated honestly.

### BreadcrumbJournal

- Purpose: resume interrupted indexing instead of starting from file 1.
- Interface: append/update active run state and completed sector checkpoints.
- Behavior: on restart, rehydrate resumable work and allow direct search to mount completed sectors immediately.

Proposed persisted record:

| Field | Meaning |
|-------|---------|
| `journal_id` | Stable identifier for the current indexing lineage |
| `run_id` | Specific process attempt writing the journal |
| `root` | Corpus root the journal belongs to |
| `requested_mode` | Search or indexing mode that initiated the run |
| `completed_sector_ids` | Ordered set of sectors already rebuilt and committed |
| `active_sector` | At most one sector currently rebuilding with a resumable cursor |
| `dirty_sector_ids` | Known-invalid sectors queued for rebuild |
| `resume_cursor` | File/member offset inside the active sector rebuild |
| `coverage_snapshot` | Last truthful `frontier`/`converging`/`sealed` view emitted to callers |
| `updated_at` | Last heartbeat from the writing process |

Recovery rules:

- If the journal is fresh and internally coherent, restart resumes from `active_sector` and mounts `completed_sector_ids` immediately.
- If the journal is stale or corrupt, discard only resumability state; keep independently valid `SectorMap` sectors mounted.
- Operators and embedders should see explicit resume signals such as `resumed_sectors`, `resume_cursor`, and `breadcrumb_recovered=false/true`.

### FrontierLedger

- Purpose: support useful search over partial coverage.
- Interface: load rolling stats/merge metadata plus coverage state.
- Behavior: allow ranking over ready sectors before a globally sealed index exists, while making coverage explicit.

Proposed persisted record:

| Field | Meaning |
|-------|---------|
| `ready_sector_ids` | Sectors currently mounted for search |
| `coverage_mode` | `frontier`, `converging`, or `sealed` |
| `doc_count` | Rolling document count across ready sectors |
| `avg_doc_len` | Rolling lexical normalization value for ready sectors |
| `term_df_sketch` | Aggregated or merged document-frequency view derived from ready sector shards |
| `missing_sector_count` | Number of sectors not yet mounted or still dirty |
| `last_converged_at` | Timestamp of the last transition toward fuller coverage |

Ranking strategy:

- `frontier` mode ranks against ready sectors only, using rolling stats over those sectors and explicit coverage signaling.
- `converging` mode uses the same mounted-sector search path, but ledger updates are still changing global-ish stats as dirty sectors finish.
- `sealed` mode freezes the ledger against a fully validated sector set and becomes equivalent to the current truthful complete-corpus contract.

### Search Coverage Mode

- Purpose: expose whether a search is `frontier`, `converging`, or `sealed`.
- Interface: progress and response metadata.
- Behavior: inform callers whether ranking is partial, converging, or fully sealed.

Mode semantics:

| Mode | Truthful meaning | Caller expectation |
|------|------------------|--------------------|
| `frontier` | Search is running only over a subset of reusable sectors; more coverage is known to be missing or dirty. | Useful early results are possible, but completeness is intentionally partial. |
| `converging` | Search is running over mounted sectors while validation/rebuild is still closing the gap to a full seal. | Results improve as more sectors converge; completeness is not final yet. |
| `sealed` | All required sectors for the current corpus proof are clean and mounted. | Equivalent to a fully validated complete-corpus search. |

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

Planned progress/reporting additions:

- `coverage_mode`
- `sectors_reused`
- `sectors_dirty`
- `sectors_rebuilt`
- `breadcrumb_resumed`
- `resume_cursor`
- `coverage_missing`
- `next_state_hint` such as `converging -> sealed`

## Data Flow

1. Startup loads `SectorMap`, `BreadcrumbJournal`, and `FrontierLedger`.
2. Search mounts any ready sectors and can begin in `frontier` or `converging` mode immediately.
3. Same-process continued validation checks sector proofs and marks sectors clean or dirty while the active run continues from ready sectors.
4. Dirty sectors rebuild their local artifacts/shards and update the ledger.
5. Coverage moves toward `sealed` as all required sectors converge.
6. Breadcrumbs are updated continuously so restart can resume without replaying the entire corpus walk.

Execution-slice decomposition:

1. `Introduce Sector Maps And Sector Hash Validity Proofs`
   - define deterministic sector boundaries
   - define the persisted `SectorMap` schema
   - define metadata-first then content-escalation validity proof rules
2. `Persist Breadcrumb Journals For Resumable Indexing`
   - define resumable sector rebuild cursors
   - define stale/corrupt journal recovery behavior
   - define operator and embedder resume signals
3. `Support Frontier Converging And Sealed Search Coverage`
   - define coverage-state truth contract
   - define rolling stats and merge behavior for mounted sectors
   - define how callers observe partial versus sealed completeness
4. `Wire Sector Reuse Into Direct Search And Progress Surfaces`
   - direct search mounts ready sectors first
   - progress surfaces report sector reuse, dirty sectors, and resume state
   - autonomous/library adoption is a later follow-on once the direct path is stable

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Sector proof missing or invalid | Cache read or proof validation fails | Mark sector dirty and exclude it from sealed reuse claims | Rebuild only the affected sector |
| Breadcrumb journal is incomplete or corrupt | Journal load/parse failure | Drop resumability for that run but keep valid sector reuse | Start a fresh indexing run from the last trusted sector state |
| Frontier ledger is unavailable | Ledger load failure | Fall back to sector-local ranking/merge strategy with explicit frontier coverage | Rebuild ledger incrementally as sectors complete |
| Coverage cannot be stated honestly | Inconsistent ready/dirty/sealed state | Downgrade reported mode to the safest truthful state | Recompute state from persisted sector and breadcrumb records |
