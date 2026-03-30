# Search Progress Callback Interface - Product Requirements

## Problem Statement

search_autonomous provides no progress visibility during execution — downstream consumers like paddles cannot show indexing, embedding, or planner step progress to users, resulting in a blocked UI with only elapsed time.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Provide phased progress callbacks from search_autonomous | Downstream consumer can display per-phase progress counters | All 5 phases emit progress events |
| GOAL-02 | Zero overhead when callback is not provided | Benchmark delta vs baseline with no callback | < 1% regression |
| GOAL-03 | Backward-compatible API — existing callers compile unchanged | Existing tests and CLI pass without modification | 0 breaking changes |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Library consumer (paddles) | TUI application embedding sift as a Rust dependency | Receive structured progress updates to render indexing/embedding/search phases |
| Existing CLI users | Current sift CLI callers | No regression — existing call sites work without changes |

## Scope

### In Scope

- [SCOPE-01] SearchProgress enum with phase-specific variants (Indexing, Embedding, Planning, Retrieving, Ranking)
- [SCOPE-02] Optional callback parameter on search_autonomous and search_autonomous_with
- [SCOPE-03] Progress emission from indexing, embedding, planner step, retrieval, and ranking phases
- [SCOPE-04] Optional estimated_remaining Duration on progress events
- [SCOPE-05] Formal upstream requirements bearing documenting paddles needs

### Out of Scope

- [SCOPE-06] Async/streaming progress (future work — this mission uses synchronous Fn callback)
- [SCOPE-07] Cancellation via callback return value (future mission)
- [SCOPE-08] Progress persistence or logging infrastructure

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Define SearchProgress enum with variants: Indexing { files_processed, files_total }, Embedding { chunks_processed, chunks_total }, PlannerStep { step_index, action, query }, Retrieving { turn_index, turns_total }, Ranking { results_processed, results_total } | GOAL-01 | must | Covers all five execution phases with phase-specific counters |
| FR-02 | Add SearchPhase enum: Indexing, Embedding, Planning, Retrieving, Ranking | GOAL-01 | must | Enables downstream phase-to-display-string mapping |
| FR-03 | Add optional estimated_remaining: Option<Duration> field to SearchProgress | GOAL-01 | should | Enables "~15s remaining" display |
| FR-04 | Add optional progress callback parameter to search_autonomous | GOAL-02, GOAL-03 | must | Callback seam for downstream consumers; Option<F> for zero-cost when absent |
| FR-05 | Emit Indexing progress from corpus loading phase | GOAL-01 | must | Reports files_processed/files_total during walkdir + artifact loading |
| FR-06 | Emit Embedding progress from vector retriever phase | GOAL-01 | must | Reports chunks_processed/chunks_total during the slow embedding step |
| FR-07 | Emit PlannerStep progress from planner trace generation | GOAL-01 | must | Reports each graph search step as it happens |
| FR-08 | Emit Retrieving and Ranking progress from search controller | GOAL-01 | must | Reports turn-by-turn retrieval and result ranking progress |
| FR-09 | Export all progress types from lib.rs public API | GOAL-03 | must | Downstream crate consumers need these types |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Zero measurable overhead when callback is None | GOAL-02 | must | Existing users must not pay for unused feature |
| NFR-02 | All existing tests pass without modification | GOAL-03 | must | Backward compatibility proof |
| NFR-03 | Progress callback is synchronous Fn, not async | GOAL-03 | must | No async runtime dependency added to sift |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Progress emission | Unit test with mock callback counting invocations per phase | Story evidence logs |
| Backward compat | Existing test suite passes (cargo nextest) | CI green |
| Zero overhead | Benchmark with/without callback | Story evidence logs |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Synchronous Fn callback is sufficient for paddles TUI | May need channel-based approach later | Confirmed by paddles requirements table |
| Corpus loading has enough granularity for file-level progress | May need to restructure walkdir loop | Inspect corpus.rs during implementation |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should callback return a ControlFlow to support cancellation? | Epic owner | Deferred to future mission |
| Does embedding phase process chunks individually or in batches? | Implementer | Open — determines counter granularity |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Downstream consumer can register a callback and receive all 5 phase progress events
- [ ] Existing CLI and tests compile and pass without changes
- [ ] Benchmark shows < 1% overhead with no callback provided
<!-- END SUCCESS_CRITERIA -->
