# True Hybrid Retrieval Architecture - Product Requirements

> Replace sift's rerank-only hybrid engine with true BM25 plus vector
> retrieval over structure-aware sections, while preserving the single-binary,
> no-daemon, no-database contract.

## Problem Statement

Sift's current hybrid engine is BM25 plus dense reranking, not BM25 plus
independent semantic vector retrieval. This misses semantically relevant whole
documents that BM25 never shortlists and underserves long structured documents
like HTML, PDF, and Office files. Sift needs a true hybrid architecture that
combines BM25 document retrieval with vector retrieval over structure-aware
sections while preserving the single-binary, no-daemon, no-database contract.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make `search --engine hybrid` a true lexical + semantic retrieval path instead of a lexical shortlist reranker. | Retrieval architecture inspection + CLI behavior | Hybrid returns documents supported by an independent vector-retrieval channel |
| GOAL-02 | Improve retrieval effectiveness on long and structured documents through section-aware semantic evidence. | Bench/eval evidence | Hybrid materially improves quality over BM25-only on the eval corpus or the shortfall is explicitly evidenced |
| GOAL-03 | Preserve local-first operating constraints while changing the retrieval architecture. | Design and implementation inspection | Single Rust binary, no daemon, no database, no persisted corpus sidecar index |
| GOAL-04 | Measure the latency cost of whole-corpus vector retrieval honestly. | Benchmark evidence | Exact commands and measured latency are recorded, with any shortfall against 200 ms explained |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Agentic Developer | Searches local docs, specs, and rich documents directly from the terminal. | Document-level results that combine exact term matching with semantic recovery |
| Coding Agent | Needs compact, semantically relevant local evidence without spinning up external services. | True hybrid retrieval that can recover relevant documents beyond lexical hits |
| Maintainer / Evaluator | Owns the product contract and benchmark evidence. | A retrieval architecture whose behavior, latency, and tradeoffs are explicit and measurable |

## Scope

### In Scope

- [SCOPE-01] Redefine the `hybrid` engine as BM25 document retrieval plus
  vector retrieval over structure-aware sections.
- [SCOPE-02] Add a structure-aware segment model that works across currently
  supported document families.
- [SCOPE-03] Aggregate segment hits into document-level semantic scores and
  snippets.
- [SCOPE-04] Replace weighted score blending with rank-based fusion for the
  hybrid result set.
- [SCOPE-05] Record new quality and latency evidence for the true-hybrid path.

### Out of Scope

- [SCOPE-06] Persisted vector sidecars or external vector databases.
- [SCOPE-07] Daemonized indexing or background workers.
- [SCOPE-08] OCR, scanned-document recovery, or LLM-guided tree search.
- [SCOPE-09] ANN acceleration as a required first slice.
- [SCOPE-10] Making `fastembed-rs` or ONNX Runtime the default runtime path in
  the first implementation.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The default `hybrid` engine must combine BM25 document retrieval with independent vector retrieval over the active corpus. | GOAL-01, GOAL-02 | must | This is the core product correction requested by the user objective. |
| FR-02 | Semantic retrieval must operate on structure-aware sections rather than single truncated whole-document embeddings. | GOAL-02 | must | Long documents need local semantic evidence to be retrievable effectively. |
| FR-03 | The final `hybrid` result set must rank documents, not raw chunks, while preserving the best section evidence for snippets and explanations. | GOAL-01, GOAL-02 | must | Sift is a document retrieval CLI, not a fragment browser. |
| FR-04 | The repository must produce benchmark evidence that compares BM25-only and true-hybrid retrieval for both quality and latency. | GOAL-02, GOAL-04 | must | Architectural changes in retrieval need measured proof, not intuition. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The default embedding and vector-retrieval path must remain local-first, single-binary, and free of resident services or external databases. | GOAL-03 | must | Preserves the repository's operating contract. |
| NFR-02 | The first implementation must avoid persisted corpus sidecar indexes unless benchmark evidence explicitly justifies changing that constraint later. | GOAL-03, GOAL-04 | must | The current board prefers transient or ephemeral retrieval structures first. |
| NFR-03 | Benchmark reports must record the exact commands, corpus shape, model settings, hardware assumptions, and measured outputs for the true-hybrid path. | GOAL-04 | must | Prevents hidden performance regressions and unverifiable claims. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Retrieval shape | unit tests + CLI proof + design inspection | Evidence that `hybrid` performs independent lexical and vector retrieval |
| Whole-document effectiveness | targeted fixture tests + eval corpus comparison | Evidence that section-backed vector retrieval contributes document-level wins |
| Constraint preservation | dependency inspection + CLI proof + code review | Evidence that the default path remains single-binary, local, and indexless |
| Performance contract | benchmark commands with exact outputs | Attached quality and latency reports with metadata |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current Candle embedding path can be reused for the first true-hybrid implementation. | We may need to broaden the runtime choice earlier than planned. | Validate during the first implementation slice and benchmarks. |
| Section-backed exact vector search is feasible for the current "thousands of files" target without persisted indices. | The design may need ANN or persistence sooner. | Benchmark latency and memory behavior during the voyage. |
| Rich-document extractors can expose enough structural signal to build useful sections. | Search quality on HTML/PDF/Office docs may remain weak. | Add format-aware fixture and benchmark coverage during execution. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much latency headroom remains after switching from reranking to whole-corpus vector retrieval? | Engineering | Open |
| Should reranking survive as an optional stage after true hybrid lands, or be removed entirely? | Engineering | Open |
| Does the first segment aggregation rule need to be format-specific, or can one general formula work across all supported document types? | Engineering | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `search --engine hybrid` uses independent BM25 and vector retrieval rather
  than reranking a lexical shortlist.
- [ ] Whole-document semantic retrieval is backed by structure-aware sections and
  returns documents with best-section snippets.
- [ ] Quality and latency evidence are recorded for the new hybrid path with
  exact commands and outputs.
- [ ] The implementation preserves the single-binary, no-daemon, no-database,
  no-persisted-index contract.
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

### Opportunity Cost

The main opportunity cost is delaying ANN tuning, a possible optional rerank
stage, and alternative runtime integrations while the team corrects the hybrid
architecture itself. That trade is justified because the current engine does not
yet satisfy the intended meaning of hybrid retrieval.

### Dependencies

The following must hold:

- sift can represent documents as structure-aware sections;
- the current Candle embedding path remains viable for the default runtime;
- benchmark work captures both quality gains and any latency regression.

### Alternatives Considered

Alternatives considered:

- Keep the current rerank-only architecture and relabel it.
  Rejected because it does not solve the user objective.
- Adopt `fastembed-rs` immediately as the default embedder.
  Deferred because the default path relies on ONNX Runtime while Candle already
  satisfies the current repo preferences.
- Use single whole-document embeddings.
  Rejected because it performs poorly on long structured documents.
- Add persisted vector sidecars immediately.
  Deferred until evidence proves they are necessary.

---

*This PRD was seeded from bearing `1vzSne000`. See `bearings/1vzSne000/` for
original research.*
