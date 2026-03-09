# Layered Search Strategy Foundation - Software Requirements Specification

> Formalize layered search execution with named strategies, BM25 baseline/champion benchmarking, and PageIndex-inspired preset composition.

**Epic:** [1vzXLN000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Introduce a search-plan domain model with explicit query
  expansion, retrieval, fusion, and reranking phases.
- [SCOPE-02] Establish DDD and hexagonal boundaries so strategy logic is owned
  by the domain/application layers and concrete algorithms sit behind ports.
- [SCOPE-03] Adapt the current BM25 and segment-vector paths into retriever
  strategies and add phrase/proximity retrieval as a first-class lexical
  strategy.
- [SCOPE-04] Introduce named strategy presets and make `hybrid` resolve through
  a configurable champion preset.
- [SCOPE-05] Extend benchmark and eval flows to compare strategies against
  BM25 and the champion with explicit metadata.

### Out of Scope

- [SCOPE-06] Full PageIndex-style consumer-agent loops, node summaries, or
  LLM-guided tree navigation.
- [SCOPE-07] Making an LLM reranker the default final layer.
- [SCOPE-08] External databases, daemonized background services, or required
  persisted sidecar indexes in this voyage.
- [SCOPE-09] Replacing the default local runtime with `fastembed-rs` or ONNX
  Runtime in this voyage.

## Assumptions & Dependencies

<!-- What we assume to be true; external systems, services, or conditions we depend on -->

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Existing BM25 and segment-vector implementations can be lifted behind retriever ports with bounded refactoring. | dependency | The voyage grows into a deeper search-engine rewrite. |
| Phrase/proximity retrieval can be implemented over the current transient corpus model without introducing persisted indexes. | assumption | A richer lexical index model may need a follow-up slice. |
| Benchmark and eval commands can reuse the shared strategy pipeline instead of keeping bespoke execution logic. | dependency | Strategy comparisons become inconsistent across commands. |
| The current pure-Rust local model path remains the default runtime during the architecture cutover. | dependency | Runtime selection would have to move into scope earlier. |

## Constraints

- Single Rust binary only.
- No external database.
- No daemon or background service.
- Default operating mode remains local-first.
- `bm25` remains the stable benchmark baseline.
- `hybrid` becomes a configurable champion alias rather than a permanently
  hard-coded algorithm.
- Domain logic should follow DDD and hexagonal boundaries so strategies,
  fusion, and reranking remain independently testable and swappable.
- Benchmark evidence must include exact commands and outputs.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Search execution SHALL be represented as a domain search plan with explicit query-expansion, retrieval, fusion, and reranking phases. | SCOPE-01 | FR-01 | unit test + design inspection |
| SRS-02 | The application layer SHALL orchestrate search plans through ports so CLI, bench, and eval adapters reuse the same strategy pipeline. | SCOPE-02 | NFR-02 | unit test + code inspection |
| SRS-03 | The query-expansion phase SHALL support zero or more query variants and SHALL be configurable per named strategy preset. | SCOPE-01 | FR-01 | unit test + CLI proof |
| SRS-04 | The retrieval phase SHALL support running multiple retrievers independently over shared prepared corpus state and collecting per-strategy candidate lists. | SCOPE-03 | FR-03 | unit test + CLI proof |
| SRS-05 | The initial lexical strategy set SHALL include BM25 as a stable baseline and at least one phrase/proximity-aware retriever. | SCOPE-03 | FR-03 | unit test + fixture proof |
| SRS-06 | The initial semantic strategy set SHALL include structure-aware segment vector retrieval as an independent retriever rather than a reranking stage. | SCOPE-03 | FR-03 | unit test + CLI proof |
| SRS-07 | The fusion phase SHALL support Reciprocal Rank Fusion as the default algorithm and SHALL retain contributor provenance for explainability and benchmarking. | SCOPE-01 | FR-01 | unit test + artifact inspection |
| SRS-08 | The reranking phase SHALL be optional and SHALL be modeled behind a reranker port with `none` as the default implementation. | SCOPE-01 | FR-06 | unit test + code inspection |
| SRS-09 | Sift SHALL ship named strategy presets including `bm25`, at least one composite hybrid preset, and one PageIndex-inspired preset definition built from available layers. | SCOPE-04 | FR-02 | CLI proof + preset inspection |
| SRS-10 | The top-level `hybrid` strategy SHALL resolve to a configurable champion preset instead of embedding retrieval logic directly in the CLI command path. | SCOPE-04 | FR-02 | unit test + CLI proof |
| SRS-11 | Bench and eval flows SHALL compare candidate strategies against both the BM25 baseline and the configured champion preset through the shared strategy pipeline. | SCOPE-05 | FR-05 | command proof |
| SRS-12 | Benchmark artifacts SHALL record strategy composition, expansion settings, fusion/reranking settings, segment configuration, model/runtime settings, corpus shape, git SHA, hardware summary, and command lines. | SCOPE-05 | NFR-03 | artifact inspection |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-13 | The default strategy stack SHALL remain single-binary, local-first, and free of external databases or resident services. | SCOPE-01 | NFR-01 | build inspection + CLI proof |
| SRS-14 | Domain search-plan logic SHALL remain independent of CLI parsing, filesystem traversal, benchmark rendering, and concrete model-runtime adapters. | SCOPE-02 | NFR-02 | code inspection + unit test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
