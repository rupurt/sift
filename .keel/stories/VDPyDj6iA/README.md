---
id: VDPyDj6iA
title: Reduce Search Pipeline Memory Allocations
type: feat
status: in-progress
created_at: 2026-03-09T17:02:00
updated_at: 2026-03-09T17:02:00
scope: VDPy8MNer/VDPyAtjbT
index: 3
started_at: 2026-03-09T17:15:35
---

# Reduce Search Pipeline Memory Allocations

## Summary

Minimize the overhead of search execution by reducing memory allocations in the core retrieval and fusion loops.

## Acceptance Criteria

- [x] [SRS-02/AC-01] `score_segments_manually` pre-allocates vectors for `SegmentHit` based on corpus size <!-- verify: manual, SRS-02:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Retrieval inner loop avoids re-allocating vectors for each retriever <!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-03] Allocation profiling shows zero allocations in the inner loop of `score_segments` <!-- verify: command, SRS-02:start:end, proof: ac-3.log -->
- [x] [SRS-02/AC-04] Total memory footprint remains stable across multiple searches in a session <!-- verify: manual, SRS-02:start:end, proof: ac-4.log -->
