# Implement Resumable Sector Rebuild Journals - Software Design Description

> Persist sector rebuild breadcrumbs so interrupted indexing resumes from the active sector and mounted clean sectors remain reusable across fresh processes.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds resumable breadcrumb persistence around sector rebuild work. It layers on the sector-validity substrate from the direct-search voyage so fresh processes can keep mounted clean sectors, resume the active dirty sector, and avoid restarting rebuild work from file 1.

## Context & Boundaries

This slice covers resumability only. It does not yet introduce frontier coverage semantics or broader runtime adoption, but it must compose cleanly with the sector cache records defined in the first voyage.

```
┌──────────────────────────────────────────────────────┐
│              Sector Rebuild Lifecycle                │
│                                                      │
│  checkpoint journal -> process interruption ->       │
│  restart load -> resume active sector                │
└──────────────────────────────────────────────────────┘
             ↑                          ↑
      sector cache substrate       direct startup path
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Sector cache substrate from voyage `VFnGWuRCh` | internal | Provides clean-sector mount state and dirty-sector work lists | planned repo contract |
| `src/search/corpus.rs` rebuild loop | internal | Checkpoint and resume dirty-sector processing | current repo |
| `src/system.rs` and progress surfaces | internal | Report resume and recovery state honestly | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Resume granularity | Resume at active sector plus cursor, not at whole-corpus level | Keeps restart work proportional to the interrupted slice. |
| Trust boundary | Sector validity and breadcrumbs stay separate | Corrupt resumability data must not invalidate independent clean-sector proofs. |
| Failure handling | Drop stale/corrupt journals conservatively while preserving clean-sector reuse | Keeps restart behavior truthful and robust. |

## Architecture

Add a breadcrumb journal record under `src/cache/` and thread it through dirty-sector rebuild orchestration in `src/search/corpus.rs`.

## Components

### BreadcrumbJournal Store

Purpose: persist the active rebuild lineage and resume cursor.

Likely files:
- `src/cache/breadcrumb.rs`
- `src/cache/mod.rs`

### Rebuild Checkpointer

Purpose: update breadcrumb state as dirty-sector work advances.

Likely files:
- `src/search/corpus.rs`

### Resume Loader

Purpose: load breadcrumb state on startup, restore active work, and preserve clean-sector mount state.

Likely files:
- `src/search/corpus.rs`
- `src/search/application.rs`

## Interfaces

Planned internal interfaces:

- breadcrumb load/save/update helpers under `src/cache/`
- dirty-sector rebuild orchestration that accepts and emits breadcrumb checkpoints
- telemetry/progress counters for resumed sectors, recovery fallback, and active resume cursor

## Data Flow

1. Startup loads sector validity state and any breadcrumb journal for the corpus root.
2. Clean sectors mount immediately from the sector cache substrate.
3. If a breadcrumb journal is present and fresh, the active dirty sector rebuild resumes from its stored cursor.
4. Rebuild checkpoints update the breadcrumb journal as files or members complete.
5. On successful completion, breadcrumb state is cleared or rolled forward to the next dirty sector.
6. If breadcrumb state is stale or corrupt, startup drops resumability but keeps independent clean-sector mounts.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Active-sector cursor is unreadable | Journal parse or cursor lookup fails | Drop resumability state for that run | Restart the dirty sector from its beginning while preserving clean-sector mounts |
| Journal heartbeat is stale | `updated_at` exceeds trust window | Treat the journal as abandoned | Resume only from trusted completed-sector state |
| Completed-sector ids disagree with sector map | Journal references unknown sector ids | Ignore untrusted breadcrumb entries | Recompute dirty work from sector validity state |
