# Progress Callback Foundation - SRS

## Summary

Epic: VFO1icY5Z
Goal: Implement SearchProgress types, callback parameter, and progress emission across all five search phases

## Scope

### In Scope

- [SCOPE-01] SearchProgress enum and SearchPhase enum type definitions, exported from lib.rs
- [SCOPE-02] Optional callback parameter on search_autonomous and search_autonomous_with
- [SCOPE-03] Progress emission from all five phases: indexing, embedding, planner steps, retrieval, ranking
- [SCOPE-04] Optional estimated_remaining Duration on all progress events
- [SCOPE-05] Formal bearing documenting upstream paddles requirements for traceability

### Out of Scope

- [SCOPE-06] Async/streaming progress channels
- [SCOPE-07] Cancellation via callback return value
- [SCOPE-08] Progress persistence or logging infrastructure

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Define SearchProgress enum with variants: Indexing, Embedding, PlannerStep, Retrieving, Ranking — each carrying phase-specific counters and estimated_remaining: Option\<Duration\> | SCOPE-01 | FR-01 | test: compile + construct all variants |
| SRS-02 | Define SearchPhase enum: Indexing, Embedding, Planning, Retrieving, Ranking with Display impl | SCOPE-01 | FR-02 | test: Display output matches expected strings |
| SRS-03 | search_autonomous accepts an optional progress callback parameter typed Option\<impl Fn(&SearchProgress)\> defaulting to None | SCOPE-02 | FR-04 | test: existing callers compile without changes |
| SRS-04 | search_autonomous_with accepts an optional progress callback parameter | SCOPE-02 | FR-04 | test: compile with and without callback |
| SRS-05 | Corpus loading emits Indexing { files_processed, files_total } after each file is loaded | SCOPE-03 | FR-05 | test: callback receives monotonically increasing files_processed up to files_total |
| SRS-06 | Embedding phase emits Embedding { chunks_processed, chunks_total } per chunk or batch | SCOPE-03 | FR-06 | test: callback receives Embedding events during vector retrieval |
| SRS-07 | Planner emits PlannerStep { step_index, action, query } for each trace step produced | SCOPE-03 | FR-07 | test: callback receives PlannerStep events matching trace step count |
| SRS-08 | Search controller emits Retrieving { turn_index, turns_total } before each turn and Ranking progress after retrieval | SCOPE-03 | FR-08 | test: callback receives Retrieving and Ranking events |
| SRS-09 | All progress types are exported from lib.rs public API | SCOPE-01 | FR-09 | test: downstream crate can import SearchProgress and SearchPhase |
| SRS-10 | Each SearchProgress variant includes estimated_remaining: Option\<Duration\> | SCOPE-04 | FR-03 | test: field present and settable on all variants |
| SRS-11 | Upstream paddles requirements are documented as a formal bearing with BRIEF, EVIDENCE, and ASSESSMENT | SCOPE-05 | FR-01 | file: bearing BRIEF.md, EVIDENCE.md, ASSESSMENT.md exist |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Zero measurable overhead when callback is None — compiler should optimize away the branch | SCOPE-02 | NFR-01 | bench: compare with/without callback on standard corpus |
| SRS-NFR-02 | All existing tests pass without modification | SCOPE-02 | NFR-02 | test: cargo nextest run |
| SRS-NFR-03 | Callback is synchronous Fn(&SearchProgress), no async runtime dependency | SCOPE-02 | NFR-03 | test: compile without tokio/async-std |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
