# Promote Autonomous Library Entry Point - Software Design Description

> Expose a supported built-in autonomous library entry point that reuses the shared planner runtime without requiring custom planner injection.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage promotes the built-in autonomous runtime into the supported
crate-root embedding surface. It focuses on API stability, documentation, and
tests rather than inventing a second execution path.

## Context & Boundaries

The bounded planner epic provides the working runtime and planner strategies.
This voyage makes that runtime the supported library-first entry point for
embedders and documents how it should be configured and consumed.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────────┐ ┌─────────────┐       │
│  │ Library     │ │ Docs +      │       │
│  │ Surface     │→│ Tests       │       │
│  └─────────────┘ └─────────────┘       │
└─────────────────────────────────────────┘
        ↑                   ↑
 [Planner Runtime]   [Embedders]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/lib.rs` | crate root | Supported embedding surface | current trunk |
| `LIBRARY.md` | repository doc | Canonical library usage guide | current trunk |
| built-in autonomous runtime | internal/runtime | Implementation behind the supported entry point | current trunk |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Public surface | Promote one built-in autonomous entry point instead of multiple variants | Keeps autonomous invocation comprehensible for embedders |
| Trace visibility | Keep planner traces in the supported response shape | Evaluation and debugging depend on inspectable planner behavior |
| Documentation boundary | Describe supported strategy and trace usage at the crate root and in `LIBRARY.md` | Avoids forcing embedders into `internal` APIs |

## Architecture

The crate root re-exports the supported autonomous entry point and planner
contracts while the implementation continues to live in the shared runtime.
Documentation and tests define the supported boundary.

## Components

- **Supported library entry point**
  Purpose: let embedders invoke autonomous planning without custom policy
  injection.
- **Library documentation**
  Purpose: explain strategy selection, traces, and configuration.
- **Regression coverage**
  Purpose: ensure autonomous and non-autonomous library modes coexist cleanly.

## Interfaces

The voyage stabilizes the crate-root autonomous API rather than introducing new
CLI-only contracts or an alternate internal-only entry point.

## Data Flow

1. Embedder calls the supported autonomous library entry point.
2. Runtime resolves the requested planner strategy.
3. Autonomous response returns planner trace plus lowered search/controller
   traces.
4. Documentation and tests assert that usage pattern as the supported shape.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Supported API drifts from runtime behavior | Library tests fail or docs go stale | Treat as contract regression | Update the surface or runtime together |
| Planner traces become internal-only | Missing trace data from the supported response | Block surface promotion | Restore planner-aware response visibility |
| Non-autonomous docs regress | Library guide loses current modes | Treat as documentation failure | Refresh the guide before rollout |
