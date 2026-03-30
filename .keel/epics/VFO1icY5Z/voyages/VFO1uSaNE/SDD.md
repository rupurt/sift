# Progress Callback Foundation - Software Design Description

> Implement SearchProgress types, callback parameter, and progress emission across all five search phases

**SRS:** [SRS.md](SRS.md)

## Overview

Thread a progress callback through the search_autonomous call stack so each execution phase can emit structured SearchProgress events to the caller. The callback is optional and typed as a generic `Fn(&SearchProgress)` parameter, allowing the compiler to monomorphize and eliminate the branch entirely when no callback is provided.

## Context & Boundaries

```
┌──────────────────────────────────────────────────────────┐
│                    search_autonomous                      │
│                                                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │ Corpus   │→ │ Planner  │→ │ Search   │→ │ Ranking  │ │
│  │ Loading  │  │ (trace)  │  │ Controller│  │          │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘ │
│       │emit         │emit         │emit          │emit   │
│       ↓             ↓             ↓              ↓       │
│  ┌──────────────────────────────────────────────────────┐ │
│  │          Option<impl Fn(&SearchProgress)>            │ │
│  └──────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
        ↑
   [Downstream: paddles TUI]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| std::time::Duration | stdlib | estimated_remaining field | stable |
| std::time::Instant | stdlib | Elapsed time tracking for estimation | stable |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Callback vs channel | Synchronous Fn callback | Simpler, zero-cost when unused, no async dep; channel can be layered on top by caller |
| Generic vs trait object | Generic `impl Fn(&SearchProgress)` | Monomorphization eliminates branch when None; trait object would add vtable cost |
| Callback wrapping | Wrap in a helper `emit(cb, progress)` function | Single call-site pattern; keeps emission one-liners throughout the codebase |
| Backward compat approach | New `_with_progress` methods + default None on existing | Existing public API unchanged; new methods for callers that need progress |

## Architecture

The callback threads through three layers:

1. **facade.rs** — Entry point adds optional callback parameter, passes to inner methods
2. **corpus.rs** — `load_search_corpus` accepts callback, emits Indexing events during walkdir
3. **planner.rs** — `plan()` accepts callback, emits PlannerStep per trace step
4. **application.rs** — `SearchService::execute()` accepts callback, emits Embedding/Retrieving/Ranking

## Components

### SearchProgress enum (domain.rs)

**Purpose:** Typed progress event for each search phase
**Interface:** Enum with five variants, each carrying phase-specific counters + estimated_remaining
**Behavior:** Immutable data carrier — no methods beyond Display

### SearchPhase enum (domain.rs)

**Purpose:** Phase discriminant for downstream display mapping
**Interface:** Five-variant enum with Display impl
**Behavior:** Returns human-readable phase names ("Indexing", "Embedding", etc.)

### emit helper (facade.rs or progress.rs)

**Purpose:** Conditionally invoke callback if Some
**Interface:** `fn emit<F: Fn(&SearchProgress)>(cb: &Option<F>, progress: SearchProgress)`
**Behavior:** No-op when None; calls callback when Some

## Interfaces

```rust
pub enum SearchPhase {
    Indexing,
    Embedding,
    Planning,
    Retrieving,
    Ranking,
}

pub enum SearchProgress {
    Indexing {
        phase: SearchPhase,
        files_processed: usize,
        files_total: usize,
        estimated_remaining: Option<Duration>,
    },
    Embedding {
        phase: SearchPhase,
        chunks_processed: usize,
        chunks_total: usize,
        estimated_remaining: Option<Duration>,
    },
    PlannerStep {
        phase: SearchPhase,
        step_index: usize,
        action: String,
        query: Option<String>,
        estimated_remaining: Option<Duration>,
    },
    Retrieving {
        phase: SearchPhase,
        turn_index: usize,
        turns_total: usize,
        estimated_remaining: Option<Duration>,
    },
    Ranking {
        phase: SearchPhase,
        results_processed: usize,
        results_total: usize,
        estimated_remaining: Option<Duration>,
    },
}
```

## Data Flow

1. Caller passes `Some(|p: &SearchProgress| { ... })` or `None` to `search_autonomous`
2. facade.rs passes callback reference to corpus loading → emits Indexing events
3. facade.rs passes callback reference to planner.plan() → emits PlannerStep events
4. facade.rs passes callback reference to search controller → emits Retrieving events
5. Search controller passes callback to SearchService::execute() → emits Embedding + Ranking events

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Callback panics | Rust unwind | Panic propagates to caller | Caller responsibility — documented in API |
| Estimated remaining is inaccurate | N/A — best-effort | Return None when insufficient data | Omit estimate for first few events until rate is stable |
