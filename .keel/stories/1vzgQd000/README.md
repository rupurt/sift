---
id: 1vzgQd000
title: Compute And Cache Embeddings On Load
type: feat
status: backlog
scope: 1vzfew000/1vzgQK000
created_at: 2026-03-09T13:05:00
updated_at: 2026-03-09T12:38:29
---

# Compute And Cache Embeddings On Load

## Context

Modify the corpus loading pipeline to ensure new documents are fully embedded before being stored in the blob cache.

## Acceptance Criteria

- [ ] [SRS-07/AC-01] Modify `load_search_corpus` to accept an optional `DenseReranker`. <!-- verify: manual, SRS-07:start:end, proof: ac-1.log -->
- [ ] [SRS-07/AC-02] In the "Slow Path" (extraction miss), use the reranker to populate segment embeddings before calling `save_blob`. <!-- verify: manual, SRS-07:start:end, proof: ac-2.log -->
- [ ] [SRS-08/AC-01] Update `SegmentVectorRetriever` to skip inference for segments that already have embeddings. <!-- verify: manual, SRS-08:start:end, proof: ac-3.log -->
