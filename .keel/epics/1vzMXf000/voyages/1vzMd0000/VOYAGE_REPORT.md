# VOYAGE REPORT: Implement Rich Document Extractors

## Voyage Metadata
- **ID:** 1vzMd0000
- **Epic:** 1vzMXf000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Add Extractor Boundary And HTML Search Support
- **ID:** 1vzMe4000
- **Status:** done

#### Summary
Introduce a shared rich-document extraction boundary beneath corpus loading and
use it to make local HTML files searchable without any preprocessing outside the
CLI.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Corpus loading routes supported files through a shared extraction boundary that returns normalized text for indexing. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test rich_document::extractor_boundary && cargo run -- search "service catalog" tests/fixtures/rich-docs --engine bm25', SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-01] `sift search` extracts searchable text from local HTML files and returns them in ranked results without external preprocessing. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test rich_document::html && cargo run -- search "html heading" tests/fixtures/rich-docs --engine bm25', SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-01] Unsupported files and extractor failures are skipped deterministically instead of crashing the overall search command. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test rich_document::skip_handling && cargo run -- search "service catalog" tests/fixtures/rich-docs --engine bm25', SRS-03:start:end, proof: ac-3.log -->

#### Implementation Insights
- **1vzMmS000: Add Formats Below The Loader Boundary**
  - Insight: Rich-format support stays tractable when the format-specific work stops at a shared extraction boundary that returns normalized text and metadata. BM25 indexing, dense reranking, CLI rendering, and benchmarks can then stay unchanged above that seam.
  - Suggested Action: Add future PDF and Office handlers by implementing the extractor path first, and only revisit ranking code if extracted text quality proves insufficient.
  - Applies To: `src/extract.rs`, `src/search.rs`, future rich-document handlers
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzMe4000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzMe4000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzMe4000/EVIDENCE/ac-3.log)

### Add Text-Bearing PDF Extraction
- **ID:** 1vzMeH000
- **Status:** done

#### Summary
Add a pure-Rust PDF extraction path for text-bearing documents and integrate it
into mixed-format search without shelling out to converters or introducing a
service dependency.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] `sift search` extracts searchable text from local text-bearing PDF files and returns PDF-backed hits in search results. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test rich_document::pdf && cargo run -- search "architecture decision" tests/fixtures/rich-docs --engine bm25', SRS-04:start:end, proof: ac-1.log -->
- [x] [SRS-05/AC-01] The PDF extraction path remains in-process and compatible with the single-binary, no-daemon architecture. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test rich_document::pdf && cargo tree | rg "pdf-extract" && cargo run -- search "architecture decision" tests/fixtures/rich-docs --engine bm25', SRS-05:start:end, proof: ac-2.log -->

#### Implementation Insights
- **1vzMmS000: Add Formats Below The Loader Boundary**
  - Insight: Rich-format support stays tractable when the format-specific work stops at a shared extraction boundary that returns normalized text and metadata. BM25 indexing, dense reranking, CLI rendering, and benchmarks can then stay unchanged above that seam.
  - Suggested Action: Add future PDF and Office handlers by implementing the extractor path first, and only revisit ranking code if extracted text quality proves insufficient.
  - Applies To: `src/extract.rs`, `src/search.rs`, future rich-document handlers
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzMeH000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzMeH000/EVIDENCE/ac-2.log)

### Add OOXML Office Extraction And Mixed-Format Verification
- **ID:** 1vzMeL000
- **Status:** done

#### Summary
Add local OOXML Office extraction for Word, Excel, and PowerPoint documents and
close the voyage with deterministic mixed-format verification artifacts.

#### Acceptance Criteria
- [x] [SRS-06/AC-01] `sift search` extracts searchable text from local `.docx`, `.xlsx`, and `.pptx` files without external converters. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test rich_document::office && cargo tree | rg "undoc" && cargo run -- search "quarterly roadmap" tests/fixtures/rich-docs --engine bm25', SRS-06:start:end, proof: ac-1.log -->
- [x] [SRS-07/AC-01] Repeated search over the same mixed-format corpus yields deterministic extraction, skip, and failure accounting. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test rich_document::determinism && cargo run -- search --json "quarterly roadmap" tests/fixtures/rich-docs --engine bm25', SRS-07:start:end, proof: ac-2.log -->
- [x] [SRS-08/AC-01] Mixed-format benchmark or report artifacts make extraction overhead and search behavior explicit for the rich-document corpus. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test bench::rich_formats && cargo run -- bench latency --engine bm25 --corpus tests/fixtures/rich-docs --queries tests/fixtures/rich-docs/test-queries.tsv && cargo run -- bench quality --engine bm25 --corpus tests/fixtures/rich-docs --qrels tests/fixtures/rich-docs/qrels/test.tsv', SRS-08:start:end, proof: ac-3.log -->

#### Implementation Insights
- **1vzN83000: Use `undoc::extract_text` as the canonical OOXML runtime path**
  - Insight: `undoc::extract_text(path)` provides a single in-process pure-Rust extraction path across Word, Excel, and PowerPoint OOXML files, which keeps the runtime aligned with the single-binary and no-daemon constraints.
  - Suggested Action: Reuse the shared extractor boundary for future rich-format handlers and prefer one canonical library entrypoint per family instead of format-specific wrappers.
  - Applies To: `src/extract.rs`, rich document ingestion, mixed-format search fixtures
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzMeL000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzMeL000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzMeL000/EVIDENCE/ac-3.log)


