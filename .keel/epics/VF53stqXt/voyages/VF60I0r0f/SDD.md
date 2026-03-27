# Hard-Cut Core Search Domain To Context Artifacts - Software Design Description

> Replace Document-centric modeling with ContextArtifact as the primary local search substrate and define shared local corpus semantics for artifact kinds.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage performs the minimum architectural cut needed to make `ContextArtifact`
the real substrate of Sift. Instead of adding another parallel model, it
replaces `Document`-centric search types with artifact-native records that can
represent the initial strictly local context kinds through one shared domain.

## Context & Boundaries

The voyage is a hard cutover of core search modeling, not a full acquisition or
controller implementation. It covers local artifact kinds only and sets the
shared semantics that later adapter and runtime work will consume.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐│
│  │Artifacts │ │IDs/Segs  │ │Metadata  ││
│  └──────────┘ └──────────┘ └──────────┘│
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/search/domain.rs` | internal module | Current search domain types to be hard-cut from `Document` to artifacts | current trunk |
| `src/cache/` | internal module | Existing cache identity and storage semantics that artifact IDs must reuse or replace cleanly | current trunk |
| `src/search/` retrievers and storage | internal modules | Retrieval and storage seams that currently assume document-centric identifiers | current trunk |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Primary domain type | Hard-cut to `ContextArtifact` | Avoids a long-lived compatibility layer and duplicated semantics |
| Artifact modeling | One top-level artifact record with kind-specific metadata | Keeps the substrate small while still supporting heterogeneous context |
| Initial scope | Local artifact kinds only | Matches the mission decision to prove the substrate without remote complexity |
| Identity model | Stable artifact IDs plus explicit segment identities | Preserves caching, deduplication, and evaluation traceability |

## Architecture

The domain shifts from `Document` as the implicit universal unit to a shared
artifact vocabulary:
- `ContextArtifact`: the primary substrate record
- `ArtifactKind`: local source classification
- `ArtifactId` and segment identity rules: stable retrieval and cache keys
- provenance/freshness/budget metadata: explicit context-management facts

Retrieval, storage, and hit-oriented types should refer to artifact identities
and metadata directly rather than through document-specific wrappers.

## Components

- Artifact domain records: represent the search unit and its kind-specific metadata.
- Artifact identity and segmentation rules: define stable IDs for whole artifacts and their segments.
- Artifact metadata envelope: carries provenance, freshness, and budget fields for downstream logic.
- Search/storage contract updates: replace document-centric references with artifact-native ones.

## Interfaces

- Core search interfaces should accept and return artifact-native records or IDs.
- Storage-facing contracts should expose artifact-oriented lookup semantics.
- Serialization should preserve the information needed by traces, tests, and future protocol emissions.

## Data Flow

1. A local source is represented as a `ContextArtifact` with a concrete `ArtifactKind`.
2. Stable artifact and segment IDs are derived for storage and retrieval reuse.
3. Provenance, freshness, and budget metadata are attached to the artifact record.
4. Retrieval/storage surfaces consume artifact-native identities instead of document-centric ones.
5. Later acquisition and context-assembly voyages reuse these semantics rather than redefining them.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Hard cut leaves `Document` as a supported parallel primary type | Compile-time or API review drift | Remove or collapse the duplicated surface | Refine cutover before planning later voyages |
| Artifact IDs are unstable across runs | Test or review detects cache/key drift | Tighten identity rules around source identity and segment ordinals | Re-run fixture proofs |
| Metadata is underspecified for downstream runtime use | Missing provenance/budget fields in review | Expand the artifact envelope before adapter work begins | Update domain contract before execution work |
| New abstractions exceed the mission's simplification goal | Design review spots graph/engine sprawl | Narrow the voyage back to artifact-only primitives | Defer generalization to a later ADR-backed slice |
