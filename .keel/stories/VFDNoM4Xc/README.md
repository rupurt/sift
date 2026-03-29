---
# system-managed
id: VFDNoM4Xc
status: done
created_at: 2026-03-28T18:17:43
updated_at: 2026-03-28T18:37:34
# authored
title: Implement Heuristic Graph Frontier Expansion
type: feat
operator-signal:
scope: VFD8ORnLV/VFD8TSJVM
index: 1
started_at: 2026-03-28T18:19:53
submitted_at: 2026-03-28T18:33:27
completed_at: 2026-03-28T18:37:34
---

# Implement Heuristic Graph Frontier Expansion

## Summary

Implement the heuristic graph planner baseline so it can expand a bounded
frontier from the root task and branch-local evidence without caller-authored
graph traces.

## Acceptance Criteria

- [x] [SRS-01/AC-01] The heuristic graph planner emits graph decisions from the root task, active frontier, and branch-local evidence without caller-authored graph traces. <!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-02/AC-02] Heuristic frontier expansion is deterministic enough to replay the same fork and selection decisions for the same input graph state. <!-- verify: manual, SRS-02:start:end -->
