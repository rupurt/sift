---
id: 1vzSwD000
title: Fuse BM25 And Vector Retrieval In Hybrid Search
type: feat
scope: 1vzSne000/1vzSy6000
status: done
created_at: 2026-03-08T22:13:53
updated_at: 2026-03-09T01:29:01
started_at: 2026-03-09T01:24:08
completed_at: 2026-03-09T01:29:01
---

# Fuse BM25 And Vector Retrieval In Hybrid Search

## Summary

Replace the current rerank-style hybrid path with BM25 document retrieval plus
vector document retrieval fused by Reciprocal Rank Fusion, and render
best-section snippets in the final document results.

## Acceptance Criteria

- [x] [SRS-05/AC-01] `search --engine hybrid` fuses independent BM25 and vector document rankings with Reciprocal Rank Fusion. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test hybrid::rrf && cargo run -- search tests/fixtures/rich-docs "architecture decision" --engine hybrid', SRS-05:start:end, proof: ac-1.log -->
- [x] [SRS-06/AC-01] Hybrid search returns document-level results with snippets sourced from the best matching segment rather than from an arbitrary whole-document truncation. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test hybrid::best_segment_snippet && cargo run -- search --json tests/fixtures/rich-docs "quarterly roadmap" --engine hybrid', SRS-06:start:end, proof: ac-2.log -->
