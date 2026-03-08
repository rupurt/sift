---
id: 1vzMe4000
title: Add Extractor Boundary And HTML Search Support
type: feat
scope: 1vzMXf000/1vzMd0000
status: backlog
created_at: 2026-03-08T15:30:44
updated_at: 2026-03-08T15:32:02
---

# Add Extractor Boundary And HTML Search Support

## Summary

Introduce a shared rich-document extraction boundary beneath corpus loading and
use it to make local HTML files searchable without any preprocessing outside the
CLI.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Corpus loading routes supported files through a shared extraction boundary that returns normalized text for indexing. <!-- verify: sh -lc 'cargo test rich_document::extractor_boundary && cargo run -- search "service catalog" tests/fixtures/rich-docs --engine bm25', SRS-01:start:end, proof: ac-1.log -->
- [ ] [SRS-02/AC-01] `sift search` extracts searchable text from local HTML files and returns them in ranked results without external preprocessing. <!-- verify: sh -lc 'cargo test rich_document::html && cargo run -- search "html heading" tests/fixtures/rich-docs --engine bm25', SRS-02:start:end, proof: ac-2.log -->
- [ ] [SRS-03/AC-01] Unsupported files and extractor failures are skipped deterministically instead of crashing the overall search command. <!-- verify: sh -lc 'cargo test rich_document::skip_handling && cargo run -- search "service catalog" tests/fixtures/rich-docs --engine bm25', SRS-03:start:end, proof: ac-3.log -->
