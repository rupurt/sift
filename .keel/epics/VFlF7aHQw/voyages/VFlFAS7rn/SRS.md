# Indexing Progress And Incremental Reuse - SRS

## Summary

Epic: VFlF7aHQw
Goal: Make direct and autonomous search visibly report indexing work and reuse persisted index artifacts when the corpus has not changed.

## Scope

### In Scope

- [SCOPE-01] Add a public direct-search progress entry point that mirrors the existing autonomous progress seam.
- [SCOPE-02] Expand search telemetry so indexing progress can report cache reuse, extraction/build work, skips, and BM25 cache/build status.
- [SCOPE-03] Share persisted BM25 preparation across direct and autonomous search flows.
- [SCOPE-04] Enable the bundled CLI to use the persisted search cache by default and render live stderr progress in text mode.

### Out of Scope

- [SCOPE-05] Background or daemonized indexing.
- [SCOPE-06] New async search APIs or cancellation semantics.
- [SCOPE-07] Redesigning the artifact cache storage format.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Add `Sift::search_with_progress` so direct search can emit `SearchProgress` callbacks just like autonomous search. | SCOPE-01 | FR-03 | test: direct search callback receives indexing/ranking progress |
| SRS-02 | Export a stable direct-search observability surface for current telemetry snapshots. | SCOPE-01, SCOPE-02 | FR-03 | test: downstream import + unit assertions over snapshot fields |
| SRS-03 | Search telemetry must record artifact extraction/build counts, skipped artifact counts, and BM25 cache/build counts in addition to the existing cache counters. | SCOPE-02 | FR-04 | test: telemetry snapshot reflects fresh-build and cache-hit paths |
| SRS-04 | Direct search must reuse a persisted BM25 index when `cache_dir` is enabled and the corpus signature is unchanged. | SCOPE-03 | FR-02 | test: repeat direct search hits BM25 cache on second run |
| SRS-05 | Autonomous/controller search must reuse the same persisted BM25 index contract when the corpus signature is unchanged. | SCOPE-03 | FR-02 | test: repeat autonomous/controller search hits BM25 cache on second run |
| SRS-06 | The bundled CLI must enable a default search cache root so repeat `sift search` runs opt into persisted reuse without extra flags. | SCOPE-04 | FR-01 | test: CLI construction path sets a cache dir |
| SRS-07 | Text-mode CLI search must render progress to stderr without polluting stdout search results. | SCOPE-04 | NFR-02 | test: renderer formatting/unit test or CLI proof |
| SRS-08 | Indexing progress output must surface file counts plus cache/build metrics that explain what work is being reused or rebuilt. | SCOPE-02, SCOPE-04 | FR-04 | test: renderer output includes cache/build metrics |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The progress callback must remain optional and impose no new required runtime dependencies. | SCOPE-01 | NFR-01 | test: existing non-progress callers compile and pass |
| SRS-NFR-02 | JSON/stdout search output must remain unchanged while progress is emitted on stderr only. | SCOPE-04 | NFR-02 | test: stdout renderer output remains stable under progress-enabled runs |
| SRS-NFR-03 | Incremental reuse must stay file-based and operate entirely under the configured cache root. | SCOPE-03, SCOPE-04 | NFR-03 | test: cache artifacts written under the configured directory |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
