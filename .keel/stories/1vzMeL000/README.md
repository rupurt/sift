---
id: 1vzMeL000
title: Add OOXML Office Extraction And Mixed-Format Verification
type: feat
scope: 1vzMXf000/1vzMd0000
status: backlog
created_at: 2026-03-08T15:31:01
updated_at: 2026-03-08T15:32:02
---

# Add OOXML Office Extraction And Mixed-Format Verification

## Summary

Add local OOXML Office extraction for Word, Excel, and PowerPoint documents and
close the voyage with deterministic mixed-format verification artifacts.

## Acceptance Criteria

- [ ] [SRS-06/AC-01] `sift search` extracts searchable text from local `.docx`, `.xlsx`, and `.pptx` files without external converters. <!-- verify: sh -lc 'cargo test rich_document::office && cargo tree | rg "undoc" && cargo run -- search "quarterly roadmap" tests/fixtures/rich-docs --engine bm25', SRS-06:start:end, proof: ac-1.log -->
- [ ] [SRS-07/AC-01] Repeated search over the same mixed-format corpus yields deterministic extraction, skip, and failure accounting. <!-- verify: sh -lc 'cargo test rich_document::determinism && cargo run -- search --json "quarterly roadmap" tests/fixtures/rich-docs --engine bm25', SRS-07:start:end, proof: ac-2.log -->
- [ ] [SRS-08/AC-01] Mixed-format benchmark or report artifacts make extraction overhead and search behavior explicit for the rich-document corpus. <!-- verify: sh -lc 'cargo test bench::rich_formats && cargo run -- bench latency --engine bm25 --corpus tests/fixtures/rich-docs --queries tests/fixtures/rich-docs/test-queries.tsv && cargo run -- bench quality --engine bm25 --corpus tests/fixtures/rich-docs --qrels tests/fixtures/rich-docs/qrels/test.tsv', SRS-08:start:end, proof: ac-3.log -->
