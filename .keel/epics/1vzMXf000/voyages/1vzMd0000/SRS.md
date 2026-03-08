# Implement Rich Document Extractors - Software Requirements Specification

> Add a shared extractor layer plus end-to-end HTML, PDF, and OOXML document support with mixed-format verification

**Epic:** [1vzMXf000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Add a shared extraction boundary that turns supported local files
  into normalized search text before transient indexing.
- [SCOPE-02] Support HTML, text-bearing PDF, and OOXML Office
  (`.docx`, `.xlsx`, `.pptx`) inputs in the local search path.
- [SCOPE-03] Prove mixed-format behavior with fixtures, command proofs, and
  benchmark/report artifacts.

### Out of Scope

- [SCOPE-04] OCR or scanned-image PDF recovery.
- [SCOPE-05] Legacy binary Office formats (`.doc`, `.xls`, `.ppt`).
- [SCOPE-06] Persisted indexing or background extraction services.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| HTML, PDF, and OOXML libraries can run in-process on Linux and macOS without shelling out | dependency | The voyage would need re-planning or narrower format scope |
| Existing search and benchmark layers can consume normalized text without ranking changes | assumption | The voyage would expand into a deeper retrieval refactor |
| Rich-format fixtures can be checked into the repo at small enough size for test use | dependency | Verification would need synthetic generators or external corpora |

## Constraints

- The result must remain a single Rust CLI binary.
- No external database, daemon, or sidecar index may be introduced.
- Extraction failures must be deterministic and observable.
- Pure-Rust runtime paths are preferred for all new format handlers.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `sift search` SHALL load supported files through a shared extraction boundary that returns normalized text and metadata for indexing. | SCOPE-01 | FR-01 | automated test + CLI demo |
| SRS-02 | `sift search` SHALL extract searchable text from local HTML files without external preprocessing. | SCOPE-02 | FR-02 | fixture test + CLI demo |
| SRS-03 | Unsupported files and extraction failures SHALL be surfaced deterministically and skipped without aborting the overall search command. | SCOPE-03 | FR-05 | automated test + CLI inspection |
| SRS-04 | `sift search` SHALL extract searchable text from local text-bearing PDF files without shelling out to an external converter. | SCOPE-02 | FR-03 | fixture test + dependency proof + CLI demo |
| SRS-06 | `sift search` SHALL extract searchable text from local OOXML Office files (`.docx`, `.xlsx`, `.pptx`) without shelling out to an external converter. | SCOPE-02 | FR-04 | fixture test + dependency proof + CLI demo |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-05 | Rich document support SHALL preserve sift's single-binary, no-daemon, no-external-database architecture. | SCOPE-01 | NFR-01 | dependency inspection + manual review |
| SRS-07 | Extraction output and skip/failure accounting SHALL be deterministic for the same mixed-format corpus. | SCOPE-03 | NFR-02 | repeatability test + manual inspection |
| SRS-08 | The voyage SHALL include mixed-format verification evidence that makes extraction overhead and search behavior explicit. | SCOPE-03 | NFR-03 | benchmark/report artifact + command proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
