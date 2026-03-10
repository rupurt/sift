---
id: VDQ0agoAl
title: Implement Mmap-Accelerated Blob Retrieval
type: feat
status: done
created_at: 2026-03-09T17:28:43
updated_at: 2026-03-09T17:09:49
scope: VDPy8MNer/VDQ0Y5HBn
index: 1
started_at: 2026-03-09T17:30:15
submitted_at: 2026-03-09T17:09:49
completed_at: 2026-03-09T17:09:49
---

# Implement Mmap-Accelerated Blob Retrieval

## Summary

Use memory-mapped I/O for retrieving documents from the global blob store to improve performance.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `load_blob` uses `memmap2` to map the document blob before deserialization <!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-02] `load_blob` falls back to standard file I/O if `mmap` fails <!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-03] Verified performance improvement for large documents via benchmarks <!-- verify: command, SRS-03:start:end, proof: ac-3.log -->
- [x] [SRS-01/AC-04] `sift search -vv` works correctly and shows expected results with mmap enabled <!-- verify: command, SRS-01:start:end, proof: ac-4.log -->
