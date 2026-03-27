# Prove Local Acquisition Adapters On The Shared Pipeline - Software Design Description

> Route strictly local context sources through explicit acquisition adapters into one shared artifact normalization, caching, and indexing pipeline.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage proves that context acquisition can be explicit without growing the
architecture. It introduces a small local adapter seam and makes current file
and project-doc ingestion use that seam, then adds the other strictly local
source classes needed for the first proof.

## Context & Boundaries

The voyage is intentionally local-only. It does not implement remote, web, or
MCP-backed adapters yet. The point is to prove that one adapter contract can
normalize local evidence into the shared artifact pipeline before optional
networked sources are considered.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐│
│  │ Adapters │ │Normalize │ │Cache/Idx ││
│  └──────────┘ └──────────┘ └──────────┘│
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| local file/project-doc extraction path | internal modules | Existing local ingestion behavior to move behind adapters | current trunk |
| artifact substrate from `VF60I0r0f` | internal contract | Shared destination model for acquired evidence | previous voyage output |
| `src/cache/` and indexing pipeline | internal modules | Shared normalization, caching, and indexing path | current trunk |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Adapter contract | One narrow local acquisition seam | Keeps source integration explicit without building a plugin framework |
| File ingestion | Existing file and project-doc paths become adapters too | Prevents a privileged legacy path from surviving the refactor |
| First slice | Local-only adapters | Reduces risk and keeps the first proof aligned with the mission constraint |
| Pipeline reuse | All local adapters feed the same normalization/cache/index path | Simplifies correctness and traceability |

## Architecture

The voyage adds a small adapter layer above the artifact preparation pipeline:
- local acquisition adapter definitions
- a coordinator or registry for selecting the correct local adapter
- normalization into shared artifact inputs
- reuse of the shared cache and indexing path

## Components

- File/project-doc adapter: acquires local repository and project-document evidence.
- Runtime-record adapter: acquires environment facts, tool outputs, and local agent-turn-style records.
- Adapter coordinator: dispatches local source descriptions to the right adapter.
- Shared preparation pipeline: normalizes adapter output, caches prepared artifacts, and updates indexes.

## Interfaces

- Adapters should return local artifact inputs plus provenance and failure metadata.
- The coordinator should expose local-only source selection without network semantics.
- Shared pipeline interfaces should remain the only supported path into cached/indexed artifact state.

## Data Flow

1. A caller identifies a strictly local source class.
2. The matching acquisition adapter reads or materializes local evidence.
3. Adapter output is normalized into the shared artifact input shape.
4. The shared preparation pipeline applies caching and indexing rules.
5. Resulting artifact state is available to later retrieval and context-assembly work.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Local source bypasses the adapter seam | Review or code-path inspection | Refactor the source behind the adapter contract | Re-run local proof |
| A second cache/index path emerges | Code review detects duplicate pipeline logic | Collapse the duplicate path into the shared preparation pipeline | Re-test shared path |
| Local runtime records over-noise the substrate | Fixture or review shows poor signal | Tighten adapter scoping and metadata | Rebalance adapter output |
| Remote semantics leak into the first voyage | Review spots network-specific logic | Defer the logic to a later slice | Keep this voyage local-only |
