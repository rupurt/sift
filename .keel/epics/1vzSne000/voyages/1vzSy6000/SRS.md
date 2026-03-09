# Structure-Aware True Hybrid Retrieval - Software Requirements Specification

> Redefine sift's hybrid engine as BM25 document retrieval plus vector retrieval over structure-aware sections, with chunk-to-document aggregation and rank-based fusion that preserve the single-binary, no-daemon, no-database contract.

**Epic:** [1vzSne000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Introduce a structure-aware segment abstraction beneath the current
  document search path.
- [SCOPE-02] Implement exact in-memory vector retrieval over all active
  segments in the corpus for the query.
- [SCOPE-03] Aggregate segment hits into document-level semantic ranks and
  best-section snippets.
- [SCOPE-04] Fuse BM25 and vector document rankings with Reciprocal Rank
  Fusion.
- [SCOPE-05] Update benchmark and eval flows to prove the new hybrid behavior
  and make its costs explicit.

### Out of Scope

- [SCOPE-06] Persisted vector sidecar indexes or external vector databases.
- [SCOPE-07] In-memory ANN as a required first implementation step.
- [SCOPE-08] LLM-guided tree search, generated summaries, or agentic retrieval
  orchestration.
- [SCOPE-09] OCR or scanned-document recovery.
- [SCOPE-10] Making `fastembed-rs` the default embedding runtime in this voyage.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The current Candle embedding path can be extended from reranking to vector retrieval over sections. | dependency | The voyage would need to broaden into an alternate runtime decision. |
| Existing extractors can expose or support enough structural signal to build useful segments. | dependency | Rich-document semantic retrieval may require a deeper extractor redesign. |
| Exact in-memory similarity search is sufficient for the first correctness-focused slice. | assumption | ANN or persistence may need to move into scope earlier. |
| The existing benchmark harness can be extended to compare BM25 and true hybrid without a new external evaluation stack. | dependency | Verification work may need its own planning slice. |

## Constraints

- Single Rust binary only.
- No external database.
- No daemon or background indexing service.
- No persisted corpus sidecar index in this voyage.
- Default search mode remains `hybrid`; the retrieval semantics change beneath
  that surface.
- Default local embedding/model execution must remain pure-Rust.
- Quality and latency evidence must include exact commands and outputs.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The search pipeline SHALL represent each input document as one or more structure-aware segments with stable document and segment identifiers. | SCOPE-01 | FR-02 | unit test + fixture inspection |
| SRS-02 | Segment construction SHALL preserve document-level search behavior while exposing section-local text that can be used for semantic retrieval and snippets. | SCOPE-01 | FR-03 | unit test + CLI proof |
| SRS-03 | The vector retrieval path SHALL score the full active corpus through segment embeddings instead of scoring only a BM25 shortlist. | SCOPE-02 | FR-01 | unit test + CLI proof |
| SRS-04 | Segment-level vector hits SHALL aggregate into document-level semantic rankings using a diminishing-returns scoring rule. | SCOPE-03 | FR-03 | unit test + fixture proof |
| SRS-05 | `search --engine hybrid` SHALL fuse BM25 document ranks and vector-derived document ranks with Reciprocal Rank Fusion. | SCOPE-04 | FR-01 | unit test + CLI proof |
| SRS-06 | Hybrid search results SHALL return documents, not raw segments, and SHALL render snippets from the best matching segment or section. | SCOPE-03 | FR-03 | unit test + CLI proof |
| SRS-07 | Benchmark and evaluation commands SHALL compare BM25-only retrieval with the true-hybrid path and report the resulting metric deltas. | SCOPE-05 | FR-04 | benchmark command proof |
| SRS-08 | Benchmark output SHALL include the segment configuration, embedding model settings, command line, git SHA, corpus shape, hardware summary, and measured fields. | SCOPE-05 | NFR-03 | benchmark artifact inspection |
| SRS-09 | The default implementation SHALL not create or require a persisted vector sidecar index or background service. | SCOPE-02 | NFR-02 | code inspection + CLI proof |
| SRS-10 | The default vector retrieval runtime SHALL remain local and pure-Rust; alternative runtimes such as `fastembed-rs` remain deferred. | SCOPE-02 | NFR-01 | dependency inspection + build verification |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-11 | True-hybrid benchmark evidence SHALL make any latency shortfall against the 200 ms target explicit rather than hiding it behind aggregate success language. | SCOPE-05 | NFR-03 | benchmark artifact inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
