# Example Consumer CLI - Software Design Description

> Provide a runnable sift-embed example crate, a just workflow to invoke it, and docs that treat it as the canonical library-consumer reference.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds a small consumer package at `examples/sift-embed` that depends
on the root `sift` crate via a path dependency and exposes a minimal
`sift-embed search [PATH] <QUERY>` command. A pair of repo-root `just` recipes
wrap the manifest-path Cargo commands so contributors can build or run the
example without remembering the plumbing. Repository docs then point at that
checked-in consumer as the canonical embedding example.

## Context & Boundaries

The example exists to prove the supported root facade is enough for another Rust
package. It is intentionally smaller than the main `sift` CLI and does not try
to expose evaluation commands, config rendering, or internal presentation
helpers.

```
┌────────────────────────────────────────────────────┐
│                    This Voyage                     │
│                                                    │
│  examples/sift-embed  just recipes  repo docs      │
│          │                 │            │           │
│          └────────────┬────┴────────────┘           │
│                       ▼                            │
│              crate-root sift facade                │
└────────────────────────────────────────────────────┘
                    ▲
               External embedders
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Root `sift` crate | local package | Supplies the supported embedded facade used by the example consumer. | path dependency |
| `clap` | library | Provides minimal CLI parsing for `sift-embed`. | current major |
| `just` | tool | Exposes repo-root helper workflows for contributors. | repository standard |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Example packaging | Use a standalone manifest under `examples/sift-embed` instead of converting the repo into a workspace. | Preserves the single-package rollout while still giving embedders a real consumer package. |
| Example command surface | Implement only `sift-embed search [PATH] <QUERY>` with current-directory defaulting. | Keeps the example focused on library consumption instead of full CLI parity. |
| Rendering | Render hits directly in the example crate instead of depending on `sift::internal` helpers. | Keeps the example on the supported embedded path. |

## Architecture

The example crate is a thin adapter:

1. Parse `search` arguments with `clap`.
2. Resolve `PATH` and `QUERY`, defaulting `PATH` to `.` when omitted.
3. Build `SearchInput` and `SearchOptions` from crate-root types.
4. Execute the search with `Sift::builder().build().search(...)`.
5. Print a compact plain-text view of the `SearchResponse`.

The root `justfile` adds helper recipes that wrap Cargo manifest-path commands
for the example crate. The README points readers to those recipes and the
example directory.

## Components

- `examples/sift-embed/Cargo.toml`: standalone package manifest with a path dependency on the root crate.
- `examples/sift-embed/src/main.rs`: minimal example CLI built on the supported facade.
- `justfile`: repo-root recipes to build and run the example package.
- `README.md` and example-local docs: guidance that positions the example as the reference consumer.

## Interfaces

- CLI: `sift-embed search [PATH] <QUERY>`
- Repo workflow: `just embed-build` and `just embed-search "<term>"`
- Library API: `Sift`, `SearchInput`, `SearchOptions`, `Retriever`, `Fusion`, `Reranking`, and result types from the crate root

## Data Flow

User input flows from the example CLI into crate-root facade types, then into
the existing search engine. Results flow back as `SearchResponse` and are
rendered locally by the example binary. The `just` recipes do not add logic;
they only forward arguments to the example package manifest path.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Example manifest or path dependency is miswired | `cargo check --manifest-path ...` fails | Treat as implementation regression | Fix manifest paths/imports |
| Example accidentally reaches into `sift::internal` | source inspection and compile review | Reject as API-boundary regression | Refactor to crate-root facade types |
| Example CLI command contract drifts from docs | story-level docs verification fails | Update docs or code in the same slice | Re-run story verification |
