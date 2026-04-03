# Incremental Indexing Visibility - Product Requirements

## Problem Statement

First-run search indexing can be slow and opaque. Sift needs stronger incremental reuse and visible progress metrics so callers and CLI users can understand cache reuse, extraction work, and BM25 index preparation while a corpus is being prepared.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make repeat searches reuse persisted indexing artifacts by default in the shipped CLI and library-owned local runtime. | Repeat search over an unchanged corpus reuses the persisted artifact cache and BM25 index instead of rebuilding both from scratch. | Cache reuse visible in proofs and tests. |
| GOAL-02 | Make indexing behavior understandable while a blocking search call is running. | Users can see file progress plus cache/build metrics that explain what the indexer is doing. | Text-mode `sift search` emits live progress on stderr and library callers can observe the same structured phases. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | Runs `sift search` against local repositories, often large enough that first-run preparation is noticeable. | Understand whether sift is reusing cached work or doing fresh extraction/index construction. |
| Embedder | Uses the crate as a blocking library call inside higher-level tooling. | Subscribe to stable progress updates without reaching into internal tracing or cache internals. |

## Scope

### In Scope

- [SCOPE-01] Enable a default persisted search cache for the bundled CLI so repeat runs reuse artifact and BM25 preparation work.
- [SCOPE-02] Reuse the persisted BM25 index in both direct and autonomous search flows when the loaded corpus signature is unchanged.
- [SCOPE-03] Expose richer indexing telemetry and render it as human-readable progress for text-mode CLI search.
- [SCOPE-04] Add a direct-search progress seam at the public library boundary so index preparation is observable outside autonomous mode too.

### Out of Scope

- [SCOPE-05] Background indexing daemons or continuously maintained sidecar databases.
- [SCOPE-06] Native streaming/async search APIs.
- [SCOPE-07] Native 1-bit or format-specific runtime acceleration work unrelated to corpus preparation.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | `sift search` must enable persisted local search caching by default so unchanged corpora can reuse prepared artifacts on repeat runs. | GOAL-01 | must | The current CLI never opts into the existing cache layer, so incremental reuse is effectively disabled for normal users. |
| FR-02 | Both direct and autonomous search paths must reuse a persisted BM25 index when the loaded corpus signature is unchanged. | GOAL-01 | must | Incremental indexing should not depend on which public search entry point the caller uses. |
| FR-03 | The public library surface must expose direct-search progress callbacks in addition to the existing autonomous progress seam. | GOAL-02 | must | Embedders need the same observability for standard blocking search that the CLI needs. |
| FR-04 | Indexing progress must make cache reuse and fresh work legible, including file counts and cache/build metrics. | GOAL-02 | must | “Indexing 37/800” alone does not explain whether sift is mostly reusing cached work or rebuilding. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The progress path must remain library-friendly: synchronous, optional, and low-overhead when unused. | GOAL-02 | must | Existing callers should not pay for progress they do not request. |
| NFR-02 | Machine-readable stdout output must remain stable; human progress rendering belongs on stderr. | GOAL-02 | must | JSON consumers cannot be forced to parse transient progress lines. |
| NFR-03 | Incremental reuse must stay file-based and local-first without introducing daemons or external databases. | GOAL-01 | must | Preserves the current architectural contract. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Direct search reuse | Tests plus CLI proof over an unchanged corpus | Story evidence showing cache-enabled repeat-run reuse |
| Autonomous reuse | Tests for controller/autonomous BM25 cache reuse | Story evidence tied to the voyage |
| Progress visibility | Unit/integration tests plus CLI proof | Story evidence showing stderr progress with cache/build metrics |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Existing per-artifact blob caching is sound enough to serve as the corpus-signature basis for BM25 reuse. | We may need a larger cache invalidation redesign. | Validate with repeat-run tests over unchanged and changed corpora. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Progress signals could become noisy if emitted too often for tiny corpora. | Engineering | Open |
| Public progress types may need additive rather than breaking evolution to stay embedding-friendly. | Engineering | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Repeat search over an unchanged corpus reuses persisted artifact preparation and BM25 index work by default in the CLI.
- [ ] Autonomous search no longer rebuilds the BM25 index on every run when the corpus signature is unchanged.
- [ ] Text-mode `sift search` shows live indexing progress that explains cache reuse versus fresh preparation work.
- [ ] Library callers can subscribe to direct-search progress without reaching into internal modules.
<!-- END SUCCESS_CRITERIA -->
