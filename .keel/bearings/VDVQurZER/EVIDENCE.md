---
id: VDVQurZER
---

# Embeddable Library Packaging Research — Evidence

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | manual | repo:file-inspection | Cargo.toml | 2026-03-10 | 2026-03-10 | high | high | The package is already named `sift` and ships both library and executable targets from one package; there is no separate CLI crate today. |
| SRC-02 | manual | repo:file-inspection | src/lib.rs | 2026-03-10 | 2026-03-10 | high | high | The top-level library exports broad internal modules (`cache`, `dense`, `eval`, `extract`, `hybrid`, `search`, `segment`, `system`, `vector`) instead of a narrow facade. |
| SRC-03 | manual | repo:file-inspection | src/search/mod.rs | 2026-03-10 | 2026-03-10 | high | high | The `search` module re-exports adapters, application, corpus, domain, and presentation internals, which turns architectural implementation detail into public API. |
| SRC-04 | manual | repo:file-inspection | src/search/domain.rs | 2026-03-10 | 2026-03-10 | high | high | Public search types carry operational details such as telemetry, cache path, query cache, model specs, and `clap::ValueEnum` derives, indicating CLI and runtime concerns leak into the library contract. |
| SRC-05 | manual | repo:file-inspection | src/main.rs | 2026-03-10 | 2026-03-10 | high | high | The executable already composes the library directly, proving embed-ability is feasible and that the missing work is boundary curation, not core extraction. |
| SRC-06 | manual | repo:file-inspection | ARCHITECTURE.md | 2026-03-10 | 2026-03-10 | medium | high | The documented architecture explicitly aims for domain isolation and hexagonal boundaries, which supports a library-first packaging direction. |
| SRC-07 | manual | repo:file-inspection | tests/performance_test.rs | 2026-03-10 | 2026-03-10 | high | high | Integration tests already use `sift` as a library (`run_search`, `SearchRequest`, `LocalFileCorpusRepository`, `DenseModelSpec`), showing the crate is usable from Rust today even if the API is low-level. |

## Technical Research

### Feasibility

Turning `sift` into an embeddable library is already technically feasible.
The package already exposes a library target, the binary depends on it, and the
integration tests consume it directly. [SRC-01] [SRC-05] [SRC-07]

The real issue is that the current library surface is accidental:

- `src/lib.rs` exposes nearly every subsystem directly. [SRC-02]
- `src/search/mod.rs` re-exports adapters and application internals that should
  not all become semver commitments. [SRC-03]
- public request and policy types leak CLI/runtime concerns such as
  `clap::ValueEnum`, telemetry, cache path management, and model/runtime
  configuration. [SRC-04]

This means the repository does not need a foundational rewrite before it can be
embedded, but it does need an API cutover.

### Option Comparison

| Option | Shape | Advantages | Costs / Risks | Assessment |
|--------|-------|------------|---------------|------------|
| A | Keep one `sift` package with both library and executable, but add a curated facade and reduce re-exports | Lowest churn, preserves current install/release story, matches current structure, fastest path to a supported API | Requires discipline to hide internals and move CLI-specific concerns out of public types | Recommended first step |
| B | Split into a workspace with a library crate and a dedicated CLI crate | Stronger dependency and semver boundary, easier to trim CLI-only dependencies from the library | More cargo/release churn, changes packaging assumptions, may be premature before the public API stabilizes | Defer until the facade proves insufficient |
| C | Introduce several internal crates (`core`, `runtime`, `models`, `cli`) immediately | Maximum architectural separation and future scalability | Highest planning and migration cost, highest risk of over-structuring before API needs are proven | Not justified yet |

### Recommended Cutover Sequence

The evidence supports a phased packaging plan:

1. Keep the current single package and executable name. [SRC-01] [SRC-05]
2. Introduce a deliberate public facade from `src/lib.rs` instead of exporting
   all internals. [SRC-02] [SRC-03]
3. Move CLI-only concerns out of public domain types, especially `clap`
   derivations and terminal rendering helpers. [SRC-03] [SRC-04] [SRC-05]
4. Add feature boundaries around heavy optional capabilities if needed
   (`eval`, rich extraction formats, LLM reranking). [SRC-01] [SRC-04]
5. Reassess whether a workspace split still adds value after the facade and
   feature boundaries exist. [SRC-03] [SRC-04]

## Key Findings

1. `sift` is already an embeddable Rust crate in practice because the package
   ships a library target and the binary/tests already consume it directly.
   [SRC-01] [SRC-05] [SRC-07]
2. The primary blocker is not feasibility but API stability: the current public
   surface is much wider and more implementation-specific than an external
   embedding contract should be. [SRC-02] [SRC-03] [SRC-04]
3. The lowest-risk path is a library-first cutover within the existing package:
   curate the facade first, then consider deeper package splits only if the
   curated API still leaves dependency or release problems unsolved. [SRC-01]
   [SRC-02] [SRC-05]
4. The repository's documented hexagonal architecture already supports this
   direction, so the recommended change is mostly packaging discipline rather
   than architectural invention. [SRC-06]

## Unknowns

- Which capabilities should become opt-in cargo features for embedders who do
  not want evaluation, rich-document extraction, or LLM reranking support?
- Whether the eventual crates.io story should keep the library and executable in
  one package permanently or promote the CLI into a separate package later.
- What the final ergonomic public API should look like: a high-level
  `SiftBuilder`/`SiftEngine` facade, lower-level search services, or both.
