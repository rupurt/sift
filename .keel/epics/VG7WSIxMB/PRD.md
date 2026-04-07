# Add Fuzzy Structural Retrieval Strategies - Product Requirements

> Add path-aware fuzzy retrieval, real structural reranking, and typo-tolerant
> fuzzy line/segment retrieval so `sift` can recover file- and symbol-shaped
> evidence that current lexical/vector search misses, especially for downstream
> `paddles` synthesis and gatherer workflows.

## Problem Statement

sift lacks path-aware fuzzy retrieval, structural reranking, and fuzzy line/segment retrieval, which leaves filename-approximate and typo-tolerant code search weaker than downstream synthesis workflows need.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Path-aware fuzzy retrieval | Approximate filename/path queries return relevant artifacts even when BM25 misses | A supported fuzzy path retriever exists and participates in fused direct search |
| GOAL-02 | Structural reranking | File, heading, and definition-shaped evidence is promoted ahead of weaker lexical ties | Structural bonuses measurably change final ordering for targeted fixture queries |
| GOAL-03 | Synthesis-friendly fuzzy content retrieval | Typo-tolerant line/segment matching returns high-signal evidence for downstream synthesis and context assembly | A fuzzy line/segment retriever exists with bounded quality filters and snippet output |
| GOAL-04 | Direct gatherer compatibility | The new retrieval capabilities stay inside the current direct-retrieval boundary that `paddles` expects | `paddles` can keep treating `sift` as direct retrieval plus evidence assembly without planner recursion |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Coding Agent | Local coding or synthesis agent searching a workspace with incomplete path or symbol recall. | Needs typo-tolerant retrieval that can still recover likely files and lines. |
| Paddles Gatherer | `paddles` direct gatherer and context assembly path consuming `sift` results. | Needs richer direct evidence without turning `sift` into a second planner. |
| Operator | CLI user iterating on code search from the terminal. | Needs approximate file and code-fragment search when exact lexical terms are unavailable. |

## Scope

### In Scope

- [SCOPE-01] A direct-search `PathFuzzyRetriever` that scores workspace artifacts by path and filename similarity.
- [SCOPE-02] A real `PositionAwareReranker` that adds structural bonuses for filename, path-component, heading, and definition-like evidence.
- [SCOPE-03] A fuzzy line/segment retriever that returns typo-tolerant snippet evidence for code-like lines and extracted segments.
- [SCOPE-04] Strategy and API surface updates so the new retrievers can be used from direct search, context assembly, and downstream library consumers.
- [SCOPE-05] Foundational documentation updates that explain the new retrieval field configuration and the `paddles` direct-retrieval boundary.

### Out of Scope

- [SCOPE-06] Editor-specific frecency, combo-memory, or current-buffer heuristics.
- [SCOPE-07] Turning `sift` into a recursive planner or autonomous gatherer for `paddles`.
- [SCOPE-08] ANN indexes, daemonized watchers, or persistent fuzzy-specific sidecar services.
- [SCOPE-09] Git-status-driven ranking as a default relevance signal.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must provide a supported path-aware fuzzy retriever that can rank artifacts from approximate filename or path queries. | GOAL-01 | must | Path-shaped recall is the most direct missing capability relative to fuzzy file finders. |
| FR-02 | The system must apply deterministic structural bonuses during reranking for filename, heading/label, and definition-shaped evidence. | GOAL-02 | must | The current position-aware reranker is nominal only; structural signal needs to affect final ordering. |
| FR-03 | The system must provide a fuzzy line/segment retrieval path that produces snippet-bearing candidates from typo-tolerant content matches. | GOAL-03 | must | Synthesis workflows need evidence-bearing lines/segments, not only file-level fuzzy recall. |
| FR-04 | The new retrieval capabilities must remain usable from the existing direct search and context assembly library surfaces that `paddles` consumes. | GOAL-04 | must | `paddles` integrates through direct retrieval and context assembly, not planner recursion. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Fuzzy retrieval quality must be bounded by explicit thresholds so weak scattered matches do not dominate direct search results. | GOAL-01, GOAL-03 | must | Approximate matching without quality gates quickly degrades trust in the result set. |
| NFR-02 | The new retrieval lanes must preserve the local-first single-binary contract and reuse the existing fusion/reranking execution path. | GOAL-01, GOAL-04 | must | The repository architecture already treats retrieval as a pluggable local field, not a service boundary. |
| NFR-03 | Documentation and public contracts must describe the new strategies in terms that remain consistent with the `paddles` direct gatherer boundary. | GOAL-04 | must | Downstream search orchestration depends on a stable understanding of what `sift` does and does not own. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Path fuzzy retrieval | Unit tests plus CLI/library proofs over path-shaped approximate queries | Story-level verification artifacts linked during execution |
| Structural reranking | Targeted ranking tests and fixture inspections | Story-level verification artifacts linked during execution |
| Fuzzy content retrieval | Unit tests and CLI/context-assembly proofs with typo-tolerant line queries | Story-level verification artifacts linked during execution |
| Downstream contract | Inspection against `paddles` request factory and gatherer boundary docs | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Path and line fuzzy retrieval can be added as retrievers inside the current fused pipeline without redesigning the planner model. | The work would sprawl into a larger search-architecture change. | Validate during implementation against the existing `SearchService` retriever path. |
| `paddles` benefits most from direct retrieval evidence richness rather than editor-memory heuristics like frecency. | We could import the wrong fuzzy-finder ideas from editor tooling. | Keep downstream checks focused on gatherer/context-assembly usage. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How aggressive should fuzzy thresholds be before approximate recall starts polluting direct-search results? | Epic owner | Open |
| Should fuzzy retrieval be enabled in the default `hybrid` preset or a dedicated champion strategy first? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Approximate filename/path queries can recover relevant artifacts through a supported fuzzy retriever.
- [ ] Structural reranking changes final ordering based on filename, heading, or definition-like evidence rather than acting as a no-op sort.
- [ ] Typo-tolerant fuzzy line/segment retrieval can return snippet-bearing evidence suitable for synthesis.
- [ ] The implementation remains consistent with the `paddles` direct-retrieval boundary.
<!-- END SUCCESS_CRITERIA -->
