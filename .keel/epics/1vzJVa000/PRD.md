# Raw Document Retrieval Architecture Research - Product Requirements

> Deliver an indexless hybrid retrieval MVP for raw local text files by
combining BM25 over transient in-memory document structures with pure-Rust dense
reranking on a bounded shortlist. Keep persisted sidecar indexes deferred unless
benchmark evidence proves they are necessary.

## Problem Statement

The repository currently has only a CLI stub plus a completed `zvec`
build-compatibility detour. That direction no longer matches the active product
contract. `sift` now needs a real end-to-end search path that:

- remains a single Rust binary
- avoids external databases, daemons, and persisted sidecar indexes
- searches raw ASCII and UTF-8 text files directly from the filesystem
- defaults to combined BM25 plus vector ranking
- uses a local pure-Rust embedding/runtime path
- proves quality and latency with reproducible benchmark evidence

The central product question is not whether we can embed a vector engine, but
whether a transient, raw-document hybrid pipeline can deliver useful retrieval
quality and acceptable latency without a persistent index.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Deliver a usable raw-document hybrid search CLI for ASCII and UTF-8 text. | End-to-end CLI proof over a multi-thousand-file corpus. | Search works without a prebuilt index. |
| GOAL-02 | Prove or falsify the latency target under the no-persisted-index constraint. | Recorded benchmark evidence with exact commands and outputs. | Under 200 ms, or a precisely measured and explained shortfall. |
| GOAL-03 | Improve ranking quality over lexical-only retrieval. | IR metrics on the evaluation corpus. | Hybrid beats BM25-only, or the gap is explicitly evidenced and re-planned. |
| GOAL-04 | Preserve the repository's local-first operating constraints. | Design and implementation inspection. | No database, no daemon, no persisted search sidecar index. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | Searches local docs or repositories from the terminal. | Fast, indexless retrieval with useful snippets. |
| Coding Agent | Needs compact, semantically relevant passages from local files. | Hybrid ranking without running an external search service. |

## Scope

### In Scope

- [SCOPE-01] Evaluation corpus tooling and benchmark harnesses for quality and
  latency.
- [SCOPE-02] Recursive raw-file search over ASCII and UTF-8 text files.
- [SCOPE-03] Hybrid ranking with BM25 over the full corpus and dense reranking
  on a bounded shortlist.
- [SCOPE-04] JSON and human-readable search output suitable for agent
  workflows.
- [SCOPE-05] Benchmark evidence that records exact commands, corpus shape,
  hardware, and measured results.

### Out of Scope

- [SCOPE-06] Persisted sidecar indexes or background indexing services.
- [SCOPE-07] PDF, HTML, and Office parsing in this epic.
- [SCOPE-08] Remote embedding APIs or non-Rust runtime dependencies.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | `sift` must search raw local text files without requiring a prebuilt or persisted search index. | GOAL-01, GOAL-04 | must | The product thesis is indexless local retrieval. |
| FR-02 | The default search path must combine BM25 and dense semantic signals into one final ranking. | GOAL-01, GOAL-03 | must | Hybrid ranking is a non-negotiable default behavior. |
| FR-03 | The repository must provide reproducible commands to download or materialize an evaluation corpus and run both quality and latency benchmarks. | GOAL-02, GOAL-03 | must | Performance and quality must be measured, not asserted. |
| FR-04 | The CLI must produce agent-friendly structured output in addition to human-readable terminal output. | GOAL-01 | should | Agentic workflows are a stated product target. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The local embedding/runtime path must remain pure Rust and work without a resident service. | GOAL-01, GOAL-04 | must | Matches the operating contract and deployment thesis. |
| NFR-02 | Benchmarks must record exact commands, hardware assumptions, and measured outputs as board evidence. | GOAL-02 | must | Avoids unverifiable performance claims. |
| NFR-03 | The MVP must target Linux and macOS for supported execution environments. | GOAL-01 | must | These platforms are required by the mission. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Use story-level CLI proofs and unit tests for filesystem traversal, document
  decoding, ranking, and output behavior.
- Use evaluation corpus commands to compare BM25-only and hybrid quality on the
  same qrels.
- Use benchmark commands to capture latency evidence with exact command lines,
  git SHA, hardware summary, and percentile measurements.
- Validate constraint compliance by code and artifact inspection: no sidecar
  index, no daemon, and pure-Rust local model execution.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A bounded dense rerank over lexical candidates is sufficient to produce measurable quality gains. | Hybrid may fail to outperform BM25-only. | Quality benchmarks on the evaluation corpus. |
| Model weight caching is acceptable as artifact caching and does not violate the "no persisted index" constraint. | Local model execution may need a different packaging strategy. | Inspect implementation and document cache behavior explicitly. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| The 200 ms target may be unattainable on CPU for some corpora without more aggressive shortlist pruning or model changes. | Implementer | Open |
| SciFact is not a code corpus and may underrepresent developer-workflow retrieval behavior. | Architect | Open |
| Hybrid quality gains may depend on chunking strategy for longer files. | Implementer | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `sift` can search a multi-thousand-file ASCII/UTF-8 corpus without a
  persisted search index.
- [ ] Default hybrid search is implemented end-to-end and evaluated against a
  BM25-only baseline.
- [ ] Exact benchmark commands and outputs are attached as evidence for both
  quality and latency.
- [ ] The implementation preserves the no-daemon, no-database, no-sidecar-index
  architecture.
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

### Opportunity Cost

The main opportunity cost is spending engineering time on benchmark and eval
infrastructure before feature breadth such as PDF, HTML, and Office ingestion.
That trade is correct because the retrieval architecture is a prerequisite for
all later format support.

### Dependencies

The following must hold:

- the CLI can materialize or ingest an evaluation corpus locally
- BM25 retrieval over raw files can be implemented without hidden persistent
  state
- a pure-Rust embedding path can load and run a small sentence-transformer
  model on CPU
- benchmark commands can be made reproducible and attached as board evidence

### Alternatives Considered

Alternatives considered:

- Keep pursuing `zvec` and disk-backed index files.
  Rejected because it conflicts with the current operating contract.
- Ship BM25-only first and defer hybrid as optional.
  Rejected because hybrid ranking is a non-negotiable default requirement.
- Compute dense scores across the full corpus on every query.
  Rejected as the least likely path to the 200 ms target without persistence.
- Start with Burn-first ONNX import.
  Deferred. Burn is viable, but direct Candle integration is a faster MVP
  route and still satisfies the pure-Rust runtime constraint.

---

*This PRD was seeded from bearing `1vzJVa000`. See `bearings/1vzJVa000/` for original research.*
