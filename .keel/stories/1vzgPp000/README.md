---
id: 1vzgPp000
title: Persist Vector Embeddings In Cache Blobs
type: feat
status: icebox
scope: 1vzfew000/1vzfjD000
created_at: 2026-03-09T13:00:00
---

# Persist Vector Embeddings In Cache Blobs

## Context

Vector retrieval is currently slow because embeddings are recomputed on every query. We need to store these embeddings in our global content-addressable blob store.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] Add `embedding: Option<Vec<f32>>` to the `Segment` struct. <!-- verify: manual, SRS-02:start:end, proof: ac-1.log -->
- [ ] [SRS-02/AC-02] Update `load_search_corpus` to compute embeddings for new/changed documents before saving the blob. <!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [ ] [SRS-02/AC-03] Ensure `SegmentVectorRetriever` uses pre-computed embeddings if available. <!-- verify: manual, SRS-02:start:end, proof: ac-3.log -->
- [ ] [SRS-02/AC-04] Update `ARCHITECTURE.md` to reflect that blobs contain fully processed assets (text + term stats + embeddings). <!-- verify: manual, SRS-02:start:end, proof: ac-4.log -->
