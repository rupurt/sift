# Adopt Sector Reuse Across Runtime Surfaces - Software Design Description

> Route controller, autonomous, and library runtime surfaces through the sector-aware preparation path and prove restart reuse end to end.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage removes the remaining runtime-surface split by routing controller and autonomous search through the same sector-aware preparation pipeline already proven in direct search. It then adds end-to-end proof coverage and operator-facing documentation so cache reuse semantics are consistent across CLI and embedded callers.

## Context & Boundaries

This is the adoption and proof voyage. It assumes sector reuse, breadcrumb resume, and coverage semantics already exist for direct search, then applies those contracts across the remaining runtime entry points without inventing a second startup authority.

```
┌──────────────────────────────────────────────────────────────┐
│                 Shared Runtime Preparation                   │
│                                                              │
│  direct/controller/autonomous -> one sector prep path        │
│          shared cache root -> reusable sector artifacts      │
└──────────────────────────────────────────────────────────────┘
             ↑                                     ↑
      previous voyages                       all runtime callers
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/facade.rs` | internal | Current split between direct and controller/autonomous startup paths to unify | current repo |
| `src/system.rs` + `src/main.rs` | internal | CLI/controller orchestration that must adopt the shared preparation path | current repo |
| Sector reuse, breadcrumb, and frontier coverage slices | internal | Existing cache and telemetry substrate that runtime adoption depends on | current repo |
| `README.md` + `LIBRARY.md` | internal | Operator and embedder docs that must describe shared cache reuse behavior | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Shared preparation owner | Centralize in the existing sift runtime/facade seam | Downstream callers should not need to care which runtime surface primed the cache. |
| Proof style | Use fresh-process end-to-end checks across surfaces | Reuse must survive process boundaries to matter operationally. |
| Docs emphasis | Describe reuse and bounded rebuild behavior, not new runtime capability | Keeps the rollout honest and aligned with the charter. |

## Architecture

Refactor runtime startup around one sector-aware preparation service:

1. direct, controller, and autonomous entry points call the same preparation seam
2. the seam returns shared sector reuse and coverage telemetry
3. runtime-specific orchestration consumes prepared search state without rebuilding separately
4. proof tests and docs validate the cross-surface contract

## Components

### Shared Preparation Seam

Purpose: own sector-aware startup for every runtime surface.

Likely files:
- `src/facade.rs`
- `src/search/application.rs`

### Runtime Surface Adapters

Purpose: update controller, autonomous, CLI, and library surfaces to consume the shared preparation result and telemetry contract.

Likely files:
- `src/system.rs`
- `src/main.rs`
- `src/facade.rs`

### Proof And Documentation Layer

Purpose: demonstrate cross-surface cache reuse and document operational expectations.

Likely files:
- integration tests under `tests/`
- `README.md`
- `LIBRARY.md`

## Interfaces

Planned interface changes:

- one runtime preparation entry point reused by direct, controller, and autonomous flows
- unified progress/telemetry contract surfaced through CLI and library APIs
- documentation updates that describe how shared cache reuse behaves across processes and surfaces

## Data Flow

1. A runtime surface requests search startup through the shared preparation seam.
2. The seam loads sector reuse state, breadcrumb resume state, and coverage metadata from the cache root.
3. Prepared sector state is returned to the runtime surface alongside coverage and reuse telemetry.
4. Later runs from a different surface reuse the same persisted sector artifacts from the same cache root.
5. Proof tests exercise direct and controller/autonomous runs across fresh processes to confirm bounded rebuilds and shared reuse.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| One runtime surface still bypasses the shared seam | Tests or code inspection find a direct whole-corpus startup call | Redirect that surface through the shared path | Keep one preparation owner in `src/facade.rs` or equivalent |
| Shared cache reuse diverges between surfaces | Cross-surface proof tests observe duplicate rebuilds | Mark the proof failing and investigate surface-specific state | Normalize runtime-specific cache root or telemetry handling |
| Docs lag the shipped behavior | Verification detects missing or stale operator guidance | Update operator and library docs in the same slice | Keep docs changes in the end-to-end proof story |
