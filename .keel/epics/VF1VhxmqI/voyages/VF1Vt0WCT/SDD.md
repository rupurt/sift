# Define Turn Model and Emission Contract - Software Design Description

> Make agentic behavior explicit as stable turn-oriented data and emission contracts without breaking the current single-turn hybrid path.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage formalizes the data model needed for agentic search before the controller is built. It introduces turn-oriented request and response contracts, explicit trace records, and emission modes that let the same retrieval substrate serve CLI views, structured protocol outputs, and latent/vector-oriented consumers.

## Context & Boundaries

The voyage changes type boundaries, not controller behavior. It should not yet introduce a multi-turn runtime; instead it creates the domain and public API surface that later controller and evaluation work will consume.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌────────────┐ ┌────────────┐ ┌──────┐│
│  │ Turn Types │ │ Emissions  │ │Facade││
│  └────────────┘ └────────────┘ └──────┘│
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/search/domain.rs` | internal module | Existing retrieval vocabulary and request/response models | current trunk |
| `src/facade.rs` | internal module | Supported crate-root API boundary | current trunk |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Turn model location | Extend the search domain rather than creating a parallel agent-only module first | Keeps the substrate shared between single-turn and multi-turn paths |
| Emission design | Model emissions explicitly rather than overloading `SearchResponse` | Prevents CLI-shaped file hits from becoming the only supported output |
| API strategy | Promote supported contracts through the facade/crate root where appropriate | The pivot needs a stable embedding surface |

## Architecture

The voyage adds a contract layer above the current retrieval pipeline:
- turn-native request/response and trace types in the domain
- emission enums or structs that describe output intent
- facade exposure for supported agentic entry points

## Components

- Turn contract types: represent controller input, per-turn state, retained evidence, and completed traces.
- Emission contracts: describe whether callers want rendered view data, structured protocol records, or latent/vector-oriented artifacts.
- Public facade surface: exposes supported constructors or methods without forcing embedders into unstable internals.

## Interfaces

- Domain interfaces should support both current file-oriented search and future turn-oriented execution.
- Public API additions should be additive where possible, preserving current `Sift::search` behavior.

## Data Flow

1. Caller constructs a turn-aware request or selects an emission mode.
2. Current retrieval substrate still performs single-turn work.
3. The response is adapted into the requested emission contract.
4. Later controller work composes multiple such turns into a trace.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Contract shape breaks current callers | Compile/test failures | Keep additive interfaces first | Refine facade cutover |
| Emission contract is underspecified | API review ambiguity | Narrow scope to explicit modes only | Expand in later voyage |
