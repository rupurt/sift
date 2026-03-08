# Build Hybrid Text Retrieval MVP - Software Requirements Specification

> Deliver raw-document ASCII and UTF-8 hybrid search, an evaluation corpus pipeline, and benchmark evidence for the indexless MVP.

**Epic:** [1vzJVa000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Provide evaluation corpus download and materialization for BEIR
  SciFact.
- [SCOPE-02] Implement raw ASCII/UTF-8 filesystem search without a persisted
  index.
- [SCOPE-03] Add BM25 baseline retrieval and pure-Rust dense reranking for the
  default search path.
- [SCOPE-04] Expose benchmark and evaluation commands that record exact
  evidence.
- [SCOPE-05] Provide human-readable and JSON result output for agent workflows.

### Out of Scope

- [SCOPE-06] PDF, HTML, and Office parsing remain out of scope for this voyage.
- [SCOPE-07] Persisted sidecar indexes and background services remain out of
  scope for this voyage.

## Assumptions & Dependencies

<!-- What we assume to be true; external systems, services, or conditions we depend on -->

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Hugging Face-hosted evaluation corpus assets remain downloadable during benchmark setup. | dependency | We need to vendor or snapshot a corpus locally instead. |
| A small sentence-transformer model can run through a pure-Rust path on CPU. | dependency | The hybrid requirement may need a different model/runtime choice. |
| ASCII and UTF-8 text decoding covers enough of the MVP corpus to produce meaningful search results. | assumption | File filtering and format support may need to expand sooner. |
| Hybrid ranking can be implemented as lexical retrieval plus dense reranking of a bounded shortlist. | assumption | We may need a different fusion or candidate-generation strategy. |

## Constraints

- Single Rust binary only.
- No external database.
- No daemon or background indexing service.
- No persisted search sidecar index in the MVP.
- Default search mode must be hybrid BM25 plus vector with combined ranking.
- Local embedding/model execution must use a pure-Rust path.
- Linux and macOS are required targets; Windows is optional for now.
- Performance evidence must include exact commands and outputs.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The repository must provide commands that download or materialize the BEIR SciFact corpus into local UTF-8 text files plus query and qrels manifests suitable for CLI benchmarking. | SCOPE-01 | FR-03 | command proof + file inspection |
| SRS-02 | The CLI must expose BM25 quality and latency benchmark commands that run against the selected corpus and emit structured output. | SCOPE-04 | FR-03 | command proof + benchmark logs |
| SRS-03 | Benchmark output must include the exact command line, git SHA, hardware summary, corpus shape, and measured fields required for reproducible evidence capture. | SCOPE-04 | NFR-02 | benchmark artifact inspection |
| SRS-04 | `sift search` must recursively scan supported ASCII/UTF-8 files from the filesystem and return ranked results without requiring a persisted index file. | SCOPE-02 | FR-01 | unit test + CLI proof |
| SRS-05 | Search results must support both human-readable output and JSON output with enough metadata for agent consumption. | SCOPE-05 | FR-04 | unit test + CLI proof |
| SRS-06 | The implementation must not create or require a persisted sidecar index or background service. | SCOPE-02 | FR-01 | code inspection + CLI proof |
| SRS-07 | The default `sift search` ranking path must combine BM25 retrieval over the full corpus with dense semantic reranking on a bounded shortlist. | SCOPE-03 | FR-02 | unit test + quality benchmark |
| SRS-08 | Dense inference must run through a pure-Rust local path suitable for Linux and macOS distribution. | SCOPE-03 | NFR-01 | dependency inspection + build verification |
| SRS-09 | The quality benchmark path must compare BM25-only and hybrid runs on the selected qrels and report the exact metric delta between them. | SCOPE-04 | FR-03 | command proof + benchmark logs |
| SRS-10 | The latency benchmark path must report p50, p90, and worst-case latency against the 200 ms target and preserve any shortfall as explicit evidence. | SCOPE-04 | NFR-02 | benchmark logs |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Pure-Rust dense inference, reproducible benchmark evidence, and explicit latency accounting are captured under SRS-03 and SRS-08 through SRS-10 for Keel story traceability. | SCOPE-03 | NFR-01 | design reference only |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
