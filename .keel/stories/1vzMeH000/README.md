---
id: 1vzMeH000
title: Add Text-Bearing PDF Extraction
type: feat
scope: 1vzMXf000/1vzMd0000
status: done
created_at: 2026-03-08T15:30:57
updated_at: 2026-03-08T15:55:55
started_at: 2026-03-08T15:51:05
completed_at: 2026-03-08T15:55:55
---

# Add Text-Bearing PDF Extraction

## Summary

Add a pure-Rust PDF extraction path for text-bearing documents and integrate it
into mixed-format search without shelling out to converters or introducing a
service dependency.

## Acceptance Criteria

- [x] [SRS-04/AC-01] `sift search` extracts searchable text from local text-bearing PDF files and returns PDF-backed hits in search results. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test rich_document::pdf && cargo run -- search "architecture decision" tests/fixtures/rich-docs --engine bm25', SRS-04:start:end, proof: ac-1.log -->
- [x] [SRS-05/AC-01] The PDF extraction path remains in-process and compatible with the single-binary, no-daemon architecture. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test rich_document::pdf && cargo tree | rg "pdf-extract" && cargo run -- search "architecture decision" tests/fixtures/rich-docs --engine bm25', SRS-05:start:end, proof: ac-2.log -->
