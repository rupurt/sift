---
id: 1vzSvm000
title: Add Structure-Aware Segment Model
type: feat
scope: 1vzSne000/1vzSy6000
status: in-progress
created_at: 2026-03-08T22:13:26
updated_at: 2026-03-09T01:06:54
started_at: 2026-03-09T01:06:54
---

# Add Structure-Aware Segment Model

## Summary

Introduce the structure-aware segment abstraction beneath the current document
loader and build source-aware segment plans for the supported document
families.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] The corpus pipeline emits stable document and segment identifiers and at least one segment for every supported searchable document. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test segment_identity && cargo test structure_aware_segments', SRS-01:start:end, proof: ac-1.log -->
- [ ] [SRS-02/AC-01] Structure-aware segments preserve section-local text that can be used later for semantic retrieval and best-section snippets across text and rich-document fixtures. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test segment_text_preservation && cargo run -- search --json tests/fixtures/rich-docs "service catalog" --engine bm25', SRS-02:start:end, proof: ac-2.log -->
