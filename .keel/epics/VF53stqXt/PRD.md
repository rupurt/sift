# Context Artifact Model And Corpus Substrate - Product Requirements

> Sift needs one canonical context unit before it needs more engine abstraction. This epic unifies files, repo docs, agent turns, tool outputs, and environment facts under one searchable substrate.

## Problem Statement

Sift lacks a single first-class context unit that can represent files, repo docs, agent turns, tool outputs, environment facts, and other evidence with shared provenance and indexing semantics.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Unified artifact model | Heterogeneous context is represented through one primary domain model instead of separate file-only and turn-only concepts | A `ContextArtifact`-style contract replaces `Document` as the primary domain type |
| GOAL-02 | Shared corpus semantics | Artifact preparation, storage, and retrieval can operate over the same substrate across source kinds | The runtime can index and retrieve more than file-backed artifacts without forking the stack |
| GOAL-03 | Explicit provenance and budget metadata | Ranking, pruning, and trace code can reason about source, freshness, and cost directly from the artifact model | Artifact records expose provenance/freshness/budget fields that are exercised in tests |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Runtime Implementer | Maintainer evolving hybrid search into a formal agentic runtime. | Needs one substrate instead of separate file and future turn stacks. |
| Embedder | Rust developer composing Sift into a larger local agent workflow. | Needs stable context records that are not tied to CLI-only file hits. |
| Evaluator | Developer comparing retrieval and context-assembly policies. | Needs provenance-rich artifacts that remain traceable and replayable. |

## Scope

### In Scope

- [SCOPE-01] Introduce artifact-domain records that can model files, repo docs, agent turns, tool outputs, environment facts, and normalized remote evidence.
- [SCOPE-02] Hard-cut the primary domain model from `Document`-centric types to artifact-native types.
- [SCOPE-03] Define shared corpus/storage semantics, identifiers, and segmentation rules for artifacts.
- [SCOPE-04] Add provenance, freshness, and budget metadata needed by ranking, pruning, and trace code.

### Out of Scope

- [SCOPE-05] A supported compatibility layer that preserves `Document` as a first-class parallel model.
- [SCOPE-06] Full acquisition-adapter implementations for every source type.
- [SCOPE-07] Full multi-turn controller policy or comparative benchmark work.
- [SCOPE-08] Premature graph-IR or Reactor generalization beyond what the artifact substrate requires.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must model context as explicit artifact records that can represent file-backed evidence, repo/project docs, agent turns, tool outputs, environment facts, and similar source material. | GOAL-01 | must | The pivot fails if every new source requires a new top-level record type. |
| FR-02 | `ContextArtifact` must become the primary domain type for the search substrate instead of remaining a parallel model beside `Document`. | GOAL-01 | must | The user explicitly wants a hard cutover and a simpler architecture. |
| FR-03 | The system must define shared corpus and storage semantics for artifacts so retrieval can operate over heterogeneous evidence without forking file-only and turn-only pipelines. | GOAL-02 | must | A codex-style substrate needs one searchable corpus concept. |
| FR-04 | Artifact records must expose provenance, freshness, and budget metadata sufficient for ranking, pruning, trace emission, and replay. | GOAL-03 | must | Controller and trace logic need explicit context facts rather than implicit prompt glue. |
| FR-05 | Artifact identifiers and segmentation rules must remain stable enough for caching, deduplication, and evaluation reuse. | GOAL-02 | must | The existing cache pipeline and future evals depend on stable identities. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The hard cutover from `Document` to `ContextArtifact` must not leave a long-lived compatibility layer in supported surfaces. | GOAL-01 | must | Temporary migration shims tend to become permanent complexity. |
| NFR-02 | The epic must avoid widening the architecture into speculative graph or engine abstractions that are not required to prove the artifact substrate. | GOAL-02 | must | Simplification is an explicit mission goal. |
| NFR-03 | Artifact records must remain serializable and inspectable for traces, tests, and downstream tooling. | GOAL-03 | must | The substrate must support replay, evaluation, and protocol emissions. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Domain model | Unit tests and API review | Story evidence proving artifact records, IDs, and metadata |
| Corpus semantics | Integration proof over heterogeneous fixtures | Story evidence showing shared preparation and retrieval paths |
| Compatibility | Regression review against current file search | Story evidence showing additive migration |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current document-oriented cache and search code can be generalized without a full rewrite. | The epic may require a deeper migration strategy. | Validate during voyage design. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which artifact segmentation rules preserve retrieval quality without exploding storage volume? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Sift has an explicit artifact model that can represent more than file-backed evidence.
- [ ] `ContextArtifact` is the primary domain type rather than a compatibility-sidecar.
- [ ] Shared corpus semantics exist for heterogeneous artifact kinds.
- [ ] Provenance, freshness, and budget metadata are available to downstream runtime code.
<!-- END SUCCESS_CRITERIA -->
