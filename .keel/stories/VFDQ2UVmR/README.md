---
# system-managed
id: VFDQ2UVmR
status: done
created_at: 2026-03-28T18:26:34
updated_at: 2026-03-28T18:37:34
# authored
title: Apply Explicit Graph Merge Semantics
type: feat
operator-signal:
scope: VFD8NgvJl/VFD8TRwUu
index: 1
started_at: 2026-03-28T18:30:07
submitted_at: 2026-03-28T18:33:27
completed_at: 2026-03-28T18:37:34
---

# Apply Explicit Graph Merge Semantics

## Summary

Support explicit merge operations so graph branches can converge through
replayable retained-evidence outcomes instead of hidden runtime repair.

## Acceptance Criteria

- [x] [SRS-01/AC-01] The runtime supports explicit graph merge semantics as a first-class operation. <!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-02/AC-02] Merge operations emit explicit branch-closure and retained-evidence outcomes that can be replayed from graph traces. <!-- verify: manual, SRS-02:start:end -->
