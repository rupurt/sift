---
id: 1vzgQX000
title: Add Embedding To Segment Model
type: feat
status: done
scope: 1vzfew000/1vzgQK000
created_at: 2026-03-09T13:05:00
updated_at: 2026-03-09T12:39:02
started_at: 2026-03-09T12:38:32
submitted_at: 2026-03-09T12:39:02
completed_at: 2026-03-09T12:39:02
---

# Add Embedding To Segment Model

## Context

Update the `Segment` struct to support persistence of pre-computed embeddings.

## Acceptance Criteria

- [x] [SRS-06/AC-01] Add `embedding: Option<Vec<f32>>` to `Segment` in `src/segment.rs`. <!-- verify: manual, SRS-06:start:end, proof: ac-1.log -->
- [x] [SRS-06/AC-02] Update `build_segments` to initialize embedding as `None`. <!-- verify: manual, SRS-06:start:end, proof: ac-2.log -->
