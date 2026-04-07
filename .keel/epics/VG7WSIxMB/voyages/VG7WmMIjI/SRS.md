# Implement Fuzzy Path And Segment Retrieval - SRS

## Summary

Epic: VG7WSIxMB
Goal: Add direct fuzzy retrieval and structural reranking that improve path-shaped recall and synthesis-ready evidence without breaking the paddles direct-search boundary.

## Scope

### In Scope

- [SCOPE-01] Add `path-fuzzy` and `segment-fuzzy` retriever policies to the shipped search plan surface and register concrete implementations in the runtime.
- [SCOPE-02] Make `PositionAwareReranker` apply deterministic structural bonuses for path, filename, heading, and definition-like evidence.
- [SCOPE-03] Update the richer built-in search presets and public `SearchPlan` helpers so structural fuzzy retrieval is available through existing direct-search APIs without adding planner behavior.
- [SCOPE-04] Document how the new structural evidence helps downstream synthesis consumers such as `paddles`.

### Out of Scope

- [SCOPE-05] Editor-memory ranking signals such as frecency, current-buffer penalties, or git-status boosts.
- [SCOPE-06] Recursive planning, cross-query fallback suggestion loops, or any runtime behavior that would make `sift` a second planner under `paddles`.
- [SCOPE-07] UI-specific picker affordances or interactive editor workflows outside the existing CLI/library surfaces.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The runtime SHALL expose a path-oriented fuzzy retriever that scores approximate filename and path-component intent and returns artifact candidates through the existing `RetrieverPolicy` and `SearchPlan` seams. | SCOPE-01 | FR-01 | automated |
| SRS-02 | The runtime SHALL expose a fuzzy line/segment retriever that can recover typo-tolerant, non-exact structural evidence and return snippet-bearing candidates suitable for downstream synthesis. | SCOPE-01 | FR-03 | automated |
| SRS-03 | `PositionAwareReranker` SHALL apply deterministic structural bonuses using artifact paths, filename stems, segment labels, and definition-like snippet features instead of acting as a score-only passthrough. | SCOPE-02 | FR-02 | automated |
| SRS-04 | The richer built-in presets and crate-root plan helpers SHALL include the structural fuzzy retrievers without changing the no-planner direct-search contract. | SCOPE-03 | FR-01 | automated |
| SRS-05 | Foundational docs SHALL explain the new retrievers, preset composition, reranker behavior, and the downstream `paddles` adoption path. | SCOPE-04 | FR-04 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Structural fuzzy retrieval SHALL remain local-first and Rust-native, without introducing external services, daemons, or non-Rust native dependencies. | SCOPE-01 | NFR-01 | automated |
| SRS-NFR-02 | New structural scores SHALL remain deterministic for a fixed corpus and query so downstream controllers and synthesis tests can reason about rank order. | SCOPE-01, SCOPE-02 | NFR-02 | automated |
| SRS-NFR-03 | The retrievers SHALL preserve snippet-bearing evidence so downstream consumers such as `paddles` can reuse them during context assembly and synthesis. | SCOPE-01, SCOPE-04 | NFR-03 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
