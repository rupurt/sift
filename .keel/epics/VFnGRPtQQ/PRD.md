# Implement Sector-Aware Frontier Search Cache Reuse - Product Requirements

## Problem Statement

Sift now has a completed sector-aware restart and frontier-search design, but the runtime still validates and rebuilds lexical state at whole-corpus granularity. We need to implement sector-scoped validity, resumable indexing, truthful partial-coverage search, and shared runtime adoption so warm restarts can return useful results without whole-corpus rescans.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Warm direct-search restarts reuse clean sectors and rebuild only dirty sectors. | First useful direct-search results no longer depend on a full workspace walk for unchanged corpora. | Unchanged-sector reuse is proven in tests and CLI evidence. |
| GOAL-02 | Interrupted indexing resumes from persisted breadcrumb state rather than restarting from file 1. | Fresh processes continue sector rebuild work from the active sector and mounted clean sectors remain usable immediately. | Restart-resume behavior is proven in tests. |
| GOAL-03 | Frontier, converging, and sealed search operate truthfully over mounted sectors with explicit coverage signaling. | Partial coverage is searchable without overstating completeness. | Coverage state is exposed and testable in progress or response contracts. |
| GOAL-04 | Controller, autonomous, CLI, and library runtime surfaces converge on one shared sector-aware preparation path. | Runtime entry points no longer diverge on whole-corpus versus sector-aware startup. | Shared-path adoption and end-to-end restart reuse are proven across shipped surfaces. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| CLI Hunter | A developer running `sift search` interactively over large local corpora. | Warm restarts that return useful results before a whole-corpus revalidation finishes. |
| Embedded Integrator | A library consumer that relies on Sift search/runtime entry points from a fresh process. | Stable sector-aware preparation and truthful coverage signaling across public surfaces. |

## Scope

### In Scope

- [SCOPE-01] Persist sector cache records, deterministic partitioning, and sector-local lexical shards on top of the existing cache root.
- [SCOPE-02] Mount clean sectors first during direct-search startup and rebuild dirty sectors in isolation.
- [SCOPE-03] Persist breadcrumb journals and resume interrupted sector rebuilds across fresh processes.
- [SCOPE-04] Implement frontier, converging, and sealed search semantics plus truthful coverage reporting.
- [SCOPE-05] Route controller, autonomous, CLI, and library runtime surfaces through the shared sector-aware preparation path and prove restart reuse end to end.

### Out of Scope

- [SCOPE-06] A daemon, external database, or background indexing service.
- [SCOPE-07] Perfect frontier-mode ranking parity with sealed mode on the first implementation slice.
- [SCOPE-08] New UI surfaces outside the existing CLI and crate-root progress/response contracts.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Direct search must persist and reuse sector-scoped validity and lexical shard state so clean sectors mount immediately on warm restart. | GOAL-01 | must | Sector reuse is the first concrete replacement for whole-corpus restart scans. |
| FR-02 | The runtime must persist breadcrumb journals and resume interrupted dirty-sector rebuilds from the active sector on the next process. | GOAL-02 | must | Resumability prevents restart-time work from repeatedly starting over. |
| FR-03 | The runtime must support truthful `frontier`, `converging`, and `sealed` coverage states with mounted-sector lexical statistics and explicit caller-visible signaling. | GOAL-03 | must | Partial search must be useful without pretending completeness. |
| FR-04 | Controller, autonomous, CLI, and library entry points must adopt the shared sector-aware preparation path and ship end-to-end restart proofs. | GOAL-04 | must | The sector-aware contract must become the runtime default rather than a one-off direct-search fork. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Preserve Sift's local-first single-binary contract while implementing sector-aware reuse. | GOAL-01, GOAL-02, GOAL-03 | must | The optimization cannot require a new operational model. |
| NFR-02 | Extend the existing manifest/blob/BM25 cache substrate instead of creating a second file-state authority. | GOAL-01, GOAL-02, GOAL-04 | must | Competing cache authorities would make restart truth hard to reason about. |
| NFR-03 | Coverage reporting and progress telemetry must remain truthful and explicit about partial versus sealed completeness. | GOAL-03 | must | Operators and embedders need to trust what kind of result they are seeing. |
| NFR-04 | Rollout must remain incremental: direct search first, then resumability, then coverage semantics, then broader runtime adoption. | GOAL-01, GOAL-04 | should | The first slice should deliver value before the full runtime migration lands. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Direct sector reuse | Unit/integration tests plus CLI restart proof | Story-level evidence showing clean-sector mount and dirty-sector rebuild isolation |
| Breadcrumb resume | Tests over interrupted and resumed runs | Story-level evidence showing active-sector resume and corrupt-journal fallback |
| Coverage semantics | Tests over frontier/converging/sealed states | Story-level evidence showing truthful progress/response signaling |
| Shared runtime adoption | Cross-surface tests and CLI/library proofs | Story-level evidence covering direct, controller, and autonomous startup |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Sector boundaries can be chosen deterministically enough that mounted sectors remain reusable across fresh processes in the common case. | Warm restart reuse may collapse back to whole-corpus behavior. | Validate in the first voyage with restart tests over unchanged and changed corpora. |
| Frontier search can rely on mounted-sector lexical statistics before a full seal exists. | Partial coverage may need a different retrieval strategy. | Validate in the frontier voyage with ranking and truthfulness tests. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| What sector partitioning strategy gives stable reuse without creating too many tiny shards? | Epic owner | Open |
| Which mounted-sector statistics are sufficient for useful frontier ranking before seal? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Warm direct-search restarts reuse clean sectors and avoid whole-corpus reparsing for unchanged corpora.
- [ ] Interrupted sector rebuilds resume from breadcrumb state on the next fresh process.
- [ ] Frontier, converging, and sealed coverage states are implemented and exposed truthfully.
- [ ] Controller, autonomous, CLI, and library surfaces share the sector-aware preparation path with end-to-end restart proofs.
<!-- END SUCCESS_CRITERIA -->
