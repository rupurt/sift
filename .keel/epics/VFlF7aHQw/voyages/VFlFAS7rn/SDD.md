# Indexing Progress And Incremental Reuse - Software Design Description

> Make direct and autonomous search visibly report indexing work and reuse persisted index artifacts when the corpus has not changed.

**SRS:** [SRS.md](SRS.md)

## Overview

Unify index preparation across direct and autonomous search so both entry points reuse the same persisted artifact and BM25 cache contract, then expose that work through a direct-search progress callback and a CLI stderr renderer. The design keeps the current synchronous callback model and extends telemetry snapshots rather than introducing a second async progress channel.

## Context & Boundaries

<!-- What's in scope, what's out of scope, external actors/systems we interact with -->

```
┌──────────────────────────────────────────────────────────┐
│                         Sift                             │
│                                                          │
│  direct search      autonomous/controller               │
│       │                     │                            │
│       └────────────┬────────┘                            │
│                    ↓                                     │
│         shared corpus + BM25 preparation                │
│                    │                                     │
│        artifact cache / manifest / bm25 cache           │
│                    │                                     │
│        progress callback + telemetry snapshot           │
└──────────────────────────────────────────────────────────┘
                    ↓
           CLI stderr progress renderer
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `std::io::IsTerminal` | stdlib | Suppress interactive progress when stderr is not a TTY | stable |
| existing `src/cache/` layer | internal module | Persist artifact and BM25 reuse state | current repo contract |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Direct progress API | Add `Sift::search_with_progress` | Mirrors the already-shipped autonomous progress shape and avoids forcing direct callers onto tracing |
| Progress detail transport | Expand telemetry snapshots rather than changing `SearchProgress` into a large index-status enum | Preserves the phase-oriented callback contract while making richer metrics accessible |
| BM25 reuse scope | Share one BM25 preparation helper across direct and autonomous paths | Removes behavioral drift between entry points |
| CLI rendering | Emit transient human progress on stderr only when stderr is a terminal | Keeps stdout stable for JSON and pipe-based consumers |

## Architecture

The voyage touches four components:

1. `facade.rs`
   Adds `search_with_progress`, exposes a telemetry snapshot accessor, and routes both direct and autonomous search through shared preparation helpers.
2. `search/application.rs`
   Introduces a shared BM25 preparation helper that can load or build the persisted index and update telemetry consistently.
3. `system.rs` + `search/domain.rs`
   Expand runtime telemetry and the public `SearchTelemetry` snapshot shape so progress consumers can tell whether work was reused or rebuilt.
4. `main.rs`
   Enables the default search cache root and renders live stderr progress for human CLI runs.

## Components

### Shared BM25 Preparation

Purpose: prepare the lexical index for a loaded corpus with cache reuse when available.

Behavior:
- compute the corpus signature from prepared artifacts
- attempt BM25 cache load when a cache root is configured
- record cache-hit versus build counters in telemetry
- emit a final indexing callback so callers can observe BM25 preparation completion

### Search Telemetry Snapshot

Purpose: provide a stable read model for current indexing/search metrics.

Behavior:
- capture counters from the internal atomic telemetry store
- expose cache reuse and fresh-work counts to CLI and embedders

### CLI Progress Renderer

Purpose: show one-line, continuously updated progress during blocking search.

Behavior:
- consume `SearchProgress` plus a fresh telemetry snapshot
- print to stderr only in interactive text mode
- summarize indexing as file progress + cache/build counters

## Interfaces

```rust
impl Sift {
    pub fn search_with_progress<F: Fn(&SearchProgress)>(
        &self,
        input: SearchInput,
        progress: Option<F>,
    ) -> Result<SearchResponse>;

    pub fn telemetry_snapshot(&self) -> SearchTelemetry;
}
```

`SearchTelemetry` remains a snapshot DTO and grows to include counts needed for incremental indexing visibility.

## Data Flow

1. CLI constructs `Sift` with a default search cache root.
2. Direct or autonomous search invokes shared corpus loading with progress.
3. Artifact loading updates telemetry for heuristic hits, blob hits, fresh extraction, skips, and total segments.
4. Shared BM25 preparation loads the cached index or builds and saves it, updating telemetry either way.
5. Progress callbacks fire through the existing phase model; the CLI renderer samples `telemetry_snapshot()` to explain what the indexing phase is doing.
6. Final search results continue to render on stdout exactly as before.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Cache directory cannot be resolved for the CLI | cache_dir helper returns error | fail search startup with actionable error | user sets `SIFT_SEARCH_CACHE` or `SIFT_CACHE` |
| BM25 cache read fails | deserialize/open error | warn via tracing, rebuild index, continue | save rebuilt index if possible |
| Progress renderer is active but stderr is redirected | `stderr().is_terminal()` false | suppress transient rendering | stdout result path still completes normally |
