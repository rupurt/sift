# Implement Sectorized Direct Search Reuse - Software Design Description

> Mount clean sectors on direct-search startup, validate them cheaply from existing cache substrate state, and rebuild only dirty sectors instead of rescanning the full corpus.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces the persisted sector cache substrate and direct-search startup path that mounts clean sectors first. It is the first executable replacement for today’s whole-corpus restart scan in `src/search/corpus.rs` and `src/search/application.rs`.

## Context & Boundaries

This slice is limited to direct-search startup and the cache records it needs. It deliberately does not implement resumable breadcrumbs, partial-coverage semantics, or controller/autonomous adoption yet.

```
┌──────────────────────────────────────────────────────────┐
│                 Direct Search Startup                    │
│                                                          │
│  Sector cache records -> clean-sector mount -> search    │
│                  dirty-sector rebuild -> save shards     │
└──────────────────────────────────────────────────────────┘
            ↑                                 ↑
     existing cache root                direct search path
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/cache/model.rs` + `src/cache/store.rs` | internal | Existing manifest/blob substrate that sector state must extend | current repo |
| `src/search/corpus.rs` | internal | Current artifact loading path to refactor into sector-aware startup | current repo |
| `src/search/application.rs` | internal | Current BM25 preparation path to refactor around sector-local shards | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Sector partition seed | Derive from corpus root plus stable partition key | Sectors must be reusable across fresh processes. |
| First shard type | Sector-local BM25 shards backed by existing artifact ids | Direct search already depends on BM25 and gives the fastest first payoff. |
| Dirty rebuild granularity | Rebuild only dirty sectors, never the whole corpus by default | This is the primary latency win the voyage is meant to ship. |

## Architecture

Add a new sector cache layer under `src/cache/` and thread it through direct-search startup:

1. sector records define membership, proof material, and lexical shard refs
2. corpus startup validates sectors from manifest heuristics first
3. clean sectors mount existing artifact + lexical state immediately
4. dirty sectors rebuild and persist refreshed shard outputs

## Components

### Sector Cache Record

Purpose: persist deterministic sector identity, proof material, and shard refs.

Likely files:
- `src/cache/sector.rs`
- `src/cache/mod.rs`

### Direct Startup Loader

Purpose: replace whole-corpus startup scanning for direct search.

Likely files:
- `src/search/corpus.rs`
- `src/search/application.rs`

Behavior:
- resolve sector records
- mount clean sectors
- schedule dirty-sector rebuilds
- hand mounted sectors to lexical preparation

### Dirty-Sector Rebuilder

Purpose: rebuild only invalid sectors and persist refreshed sector-local shards.

Likely files:
- `src/search/corpus.rs`
- `src/search/application.rs`

## Interfaces

Planned internal interfaces:

- sector record load/save helpers under `src/cache/`
- direct corpus preparation path that returns mounted clean sectors plus dirty-sector rebuild work
- sector-local lexical shard save/load helpers keyed by `sector_id`

## Data Flow

1. Direct search startup loads sector records for the corpus root.
2. For each sector, startup checks manifest/file heuristics and existing shard refs.
3. Clean sectors mount immediately into the prepared lexical corpus.
4. Dirty sectors rebuild their artifacts and lexical shard in isolation.
5. Rebuilt sectors replace stale refs in the persisted sector map.
6. Search runs over the mounted sector set without first reparsing every unchanged file.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Sector record missing for a corpus | No persisted sector metadata exists | Fall back to initial sector build path | Persist sector records as part of the first successful run |
| Sector shard missing but proofs say clean | Cache refs missing or unreadable | Mark only that sector dirty | Rebuild the missing sector shard |
| Partitioning rules change incompatibly | Sector ids no longer match persisted data | Invalidate prior sector map version | Rebuild sectors under the new schema version |
