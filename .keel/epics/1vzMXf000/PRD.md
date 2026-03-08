# Rich Document Extraction Support - Product Requirements

> Sift can extend beyond raw text files without violating the single-binary,
no-daemon, no-database contract by adding Rust-native extraction adapters for
HTML, PDF, and modern Office documents. If the extraction path is good enough
to recover stable plain-text search corpora on demand, the existing transient
BM25 + hybrid ranking pipeline can stay unchanged above the loader boundary.

## Problem Statement

The MVP only supports ASCII and UTF-8 text files. Real developer corpora
regularly include local HTML exports, API docs, PDFs, and OOXML Office files.
Without native extraction support, sift misses a large share of relevant
documents and forces users back to ad hoc conversion steps or separate tools.

The main risk is choosing format handlers that quietly violate the repository
constraints by pulling in native runtimes, OCR stacks, or service-like
behavior. PDF and Office extraction are also quality-sensitive: poor text
recovery or layout loss can degrade both lexical and dense retrieval.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Search mixed local corpora containing HTML, PDF, and OOXML Office files without pre-conversion | Format coverage | End-to-end search works across all targeted formats |
| GOAL-02 | Preserve sift's lightweight local architecture while adding extraction support | Architecture compliance | Single binary, no daemon, no external database, pure-Rust path maintained |
| GOAL-03 | Keep extraction behavior measurable and debuggable | Verification coverage | Each new format has fixtures, deterministic failure handling, and benchmark evidence |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Agentic Developer | Uses sift inside local coding and debugging workflows | Search documentation and reports without manually converting formats |
| Tooling Engineer | Maintains repository-local search and automation flows | Keep extraction local, deterministic, and easy to package |

## Scope

### In Scope

- [SCOPE-01] Add a document extraction boundary beneath corpus loading.
- [SCOPE-02] Support local HTML files through Rust-native HTML-to-text conversion.
- [SCOPE-03] Support text-bearing PDFs through a pure-Rust extraction path.
- [SCOPE-04] Support modern OOXML Office files (`.docx`, `.xlsx`, `.pptx`) through a local Rust library path.
- [SCOPE-05] Extend mixed-format verification, fixtures, and benchmarks so new format coverage is proven.

### Out of Scope

- [SCOPE-06] OCR or scanned-image PDF recovery.
- [SCOPE-07] Legacy binary Office formats (`.doc`, `.xls`, `.ppt`).
- [SCOPE-08] Any daemonized conversion service, external database, or persisted sidecar index.
- [SCOPE-09] Unrelated ranking experiments that do not materially affect rich document ingestion.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Sift SHALL normalize supported file types through a shared extractor boundary before indexing and ranking. | GOAL-01, GOAL-03 | must | Keeps search logic format-agnostic while allowing more formats to plug in safely. |
| FR-02 | Sift SHALL extract searchable text from local HTML files without requiring preprocessing outside the CLI. | GOAL-01 | must | HTML is the lowest-risk rich-document format and broadens immediate corpus coverage. |
| FR-03 | Sift SHALL extract searchable text from local text-bearing PDF files through a pure-Rust path. | GOAL-01, GOAL-02 | must | PDF support is a major corpus-coverage gain and must preserve local-runtime constraints. |
| FR-04 | Sift SHALL extract searchable text from local OOXML Office files (`.docx`, `.xlsx`, `.pptx`) through a pure-Rust path. | GOAL-01, GOAL-02 | must | Office documents are common in real working sets and need first-class local support. |
| FR-05 | Sift SHALL surface unsupported or failed extractions deterministically instead of crashing or silently corrupting the corpus view. | GOAL-03 | must | Mixed-format ingestion needs explicit operator feedback to stay debuggable. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Rich document support SHALL preserve the single-binary, no-daemon, no-external-database architecture. | GOAL-02 | must | This is a non-negotiable repository constraint. |
| NFR-02 | Extraction output and skip/failure handling SHALL be deterministic for the same input corpus. | GOAL-03 | must | Search and benchmark evidence must remain reproducible. |
| NFR-03 | Rich document rollout SHALL include benchmark or corpus evidence that makes extraction overhead and search behavior explicit. | GOAL-03 | must | Format support should be added with measured tradeoffs, not hidden costs. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove functional behavior through story-level verification evidence mapped to voyage requirements.
- Validate non-functional posture with operational checks and documented artifacts.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| HTML, PDF, and OOXML corpus coverage will improve real recall enough to justify extraction complexity | Epic could be over-scoped | Validate with mixed-format fixtures and benchmark evidence in voyages |
| Rust-native libraries are sufficient for the first rollout without shelling out to converters | Scope may need re-planning | Verify dependency/runtime shape during implementation stories |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| PDF extraction quality may vary by layout and text encoding quality | Engineering | Open |
| OOXML extraction dependency choice may need revision if output quality or compile cost is poor | Engineering | Open |
| Mixed-format loading may increase search latency enough to require a format-aware loading strategy | Engineering | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [x] Identify a plausible Rust-native extraction path for HTML, PDF, and Office formats that keeps sift as a single local binary with no daemon or external database.
- [x] Recommend an implementation sequence, explicit format boundaries, and known limitations that can be turned directly into the next epic, voyage, and stories.
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

### Opportunity Cost

Pursuing rich document support delays deeper ranking work such as chunking,
query-time caching, and more aggressive hybrid fusion tuning. That tradeoff is
acceptable because unsupported formats are a larger practical recall gap than
another small ranking iteration on text-only corpora.

### Dependencies

The following need to hold:

- The chosen extraction crates must stay compatible with Linux and macOS in a
  pure-Rust, in-process path.
- Sift needs a narrow extractor interface so new formats do not fork the search
  or benchmark codepaths.
- The next stories must include local fixtures and evidence for extraction
  correctness, not just happy-path parsing.

### Alternatives Considered

Alternatives considered:

- Stay text-only for longer. Rejected because the board has already finished the
  text-only MVP and the user objective explicitly continues into richer formats.
- Shell out to system converters such as LibreOffice, Pandoc, or PDF toolkits.
  Rejected because this weakens the single-binary/local-runtime contract.
- Hand-roll every extractor from ZIP/XML/PDF primitives. Rejected as the first
  move because it increases implementation cost without evidence that the
  existing Rust libraries are inadequate.

---

*This PRD was seeded from bearing `1vzMXf000`. See `bearings/1vzMXf000/` for original research.*
