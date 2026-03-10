# Library Facade and Packaging Cutover - Software Design Description

> Define the first supported embedded API and packaging boundary while preserving the current executable contract

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage keeps the current single-package `sift` crate but changes what is
considered public and supported.

The implementation direction is:

1. define a small canonical embedded API at the top level of the library
2. keep existing search/corpus/runtime modules available for internal use
3. move CLI-only concerns out of the supported library contract
4. have `src/main.rs` consume the same facade that embedders will use

The design intentionally defers any workspace split until the new facade exists
and can be judged on real implementation friction.

## Context & Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│                     This Voyage                             │
│                                                             │
│  Public Facade  ->  Internal Search Runtime  ->  Corpus     │
│       ^                       ^                    Cache     │
│       |                       |                              │
│    Rust embedder         Existing modules                   │
│       |                                                     │
│       +-------------------- CLI ----------------------------+│
└─────────────────────────────────────────────────────────────┘

External exclusions:
- workspace/package split
- search-quality changes
- new model/extraction capabilities
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing `src/search/*` services | internal module | Search execution engine behind the facade | current repo |
| Existing `src/main.rs` CLI | internal adapter | Consumer that proves the facade works for executable usage too | current repo |
| Cargo package metadata | packaging boundary | Preserve current package/release/install contract | current repo |
| Rust test and command verification | verification | Validate both library and CLI paths | `cargo test`, `cargo run` |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Packaging strategy | Keep one `sift` package first | Lowest-churn path that matches current repo structure and release contract |
| Public API shape | Add one canonical facade instead of re-exporting broad internals | Prevents accidental semver commitment to implementation details |
| CLI boundary | Move CLI-only parsing/rendering concerns out of supported public types | Embedded consumers should depend on search concepts, not terminal glue |
| Workspace split | Defer | Decide only after the facade exposes real packaging pressure |

## Architecture

The cutover introduces three layers:

- Public facade:
  exposes the supported embedded API, likely via a builder/service/options
  surface such as `Sift`, `SiftBuilder`, `SearchOptions`, and `SearchResults`
- Internal runtime:
  keeps `search`, `cache`, `dense`, `extract`, and related modules available
  for implementation, but not all of them remain top-level supported exports
- CLI adapter:
  parses command-line input, loads config, formats terminal output, and calls
  the public facade rather than reaching into internals directly

## Components

### Public Facade

Purpose:
provide the stable entrypoint embedders use.

Behavior:
- accepts high-level search inputs
- owns default wiring for corpus loading and search execution
- returns library-oriented results rather than terminal formatting

### Internal Search Runtime

Purpose:
retain the current search implementation and registries without turning every
module into supported API.

Behavior:
- executes retrieval, fusion, reranking, and snippet resolution
- remains free to evolve behind the facade

### CLI Adapter

Purpose:
keep executable behavior intact while proving the facade is sufficient.

Behavior:
- translates CLI/config inputs into facade options
- renders terminal output outside the core embedded API

## Interfaces

Expected interface direction:

- high-level constructor/builder for default runtime wiring
- search request/options type without CLI derives
- search response/results type suitable for both CLI and library callers
- documentation examples that show the supported dependency and invocation path
  for Rust embedders
- explicit internal modules that stay non-canonical even if still `pub(crate)`
  or otherwise available for tests/internal wiring

## Data Flow

1. Rust embedder or CLI builds the public facade.
2. Caller submits path/query/options through the facade.
3. Facade delegates to the existing corpus repository and search runtime.
4. Runtime returns library-oriented results.
5. CLI optionally renders those results into terminal output.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Facade cannot represent a needed internal capability | Integration tests or CLI rewiring becomes awkward | Expand the facade deliberately or re-plan the voyage | Prefer extending the facade over re-exposing internals wholesale |
| CLI behavior changes accidentally during rewiring | Help-text or command proofs fail | Treat as regression and fix before acceptance | Keep CLI verification in every relevant story |
| Export trimming breaks current internal callers/tests | Rust compilation or tests fail | Add explicit internal imports or narrower re-exports | Preserve one canonical public path without breaking internal development |
