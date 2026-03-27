# Expose Artifact-Based Context Assembly And Emissions - Software Design Description

> Assemble bounded local context over artifacts and expose simplified CLI and library outputs without a Document compatibility surface.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces the smallest runtime surface that makes the substrate
useful: assemble bounded local context over artifacts and emit that result in
visual, protocol, or latent forms. The goal is to simplify the runtime, not to
generalize it into a graph framework.

## Context & Boundaries

The voyage stops short of full controller policy. It defines the assembly and
emission surface that later runtime work can call. The current `search` path
should remain available as a wrapper or preserved mode, but the new contract is
artifact-native and does not keep a supported `Document` sidecar.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐│
│  │Assembly  │ │Budgets   │ │Emissions ││
│  └──────────┘ └──────────┘ └──────────┘│
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| artifact substrate from `VF60I0r0f` | internal contract | Primary domain type and metadata for context assembly | previous voyage output |
| local acquisition pipeline from `VF60I100k` | internal runtime | Supplies prepared local artifacts | previous voyage output |
| current CLI/facade surfaces | internal modules | Existing entry points to wrap or evolve | current trunk |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Runtime surface | Add an explicit artifact-based context-assembly contract | Smaller and clearer than overloading `search` or introducing a public graph API |
| Emission model | Make visual/protocol/latent outputs explicit result shapes | Prevents `SearchResponse` from remaining the only supported output contract |
| Compatibility strategy | No supported `Document` runtime sidecar | Keeps the simplification honest |
| CLI preservation | Keep current single-turn hybrid UX through a wrapper or preserved mode | Avoids regressing shipped behavior during the cutover |

## Architecture

The voyage introduces three small layers:
- context-assembly request/response contracts over artifacts
- explicit budget and retention/pruning inputs plus result traces
- emission result shapes for visual, protocol, and latent consumers

CLI and library surfaces should consume these contracts directly or wrap them,
not bypass them.

## Components

- Assembly contract types: define what artifacts, budget, and assembly mode are requested.
- Assembly runtime: selects and curates bounded artifact context.
- Emission contract types: represent visual, protocol, and latent outputs explicitly.
- Public surfaces: expose the simplified runtime to CLI and embedders.

## Interfaces

- Assembly request should accept artifact-oriented inputs and explicit budget policy.
- Assembly response should expose retained artifacts, pruning outcomes, and an emission payload.
- CLI/facade entry points should avoid requiring graph configuration or `Document` compatibility types.

## Data Flow

1. Caller submits an artifact-based assembly request.
2. Runtime applies bounded retention/pruning policy over available local artifacts.
3. Assembly result records retained/pruned context and budget outcomes.
4. Result is emitted in the requested visual, protocol, or latent form.
5. CLI or facade layers render or forward the emission without rebuilding a parallel response model.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Assembly contract still depends on `Document` compatibility types | API review or compile drift | Remove the compatibility surface before planning | Tighten contract cutover |
| Output contract remains CLI-shaped only | Type review finds `SearchResponse` coupling | Extract explicit emission result shapes | Re-run surface review |
| Budget behavior is opaque | Missing retention/pruning fields in result | Expand assembly result contract | Revisit before controller integration |
| Public surface grows toward graph/Reactor complexity | Review detects premature abstraction | Collapse to the minimal assembly-oriented API | Defer generalization |
