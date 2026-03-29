---
# system-managed
id: VFDQ2Vkov
status: done
created_at: 2026-03-28T18:26:34
updated_at: 2026-03-28T18:37:46
# authored
title: Report Graph Metrics And Comparative Deltas
type: feat
operator-signal:
scope: VFD8P8CO4/VFD8TTCXY
index: 2
started_at: 2026-03-28T18:30:07
submitted_at: 2026-03-28T18:33:27
completed_at: 2026-03-28T18:37:46
---

# Report Graph Metrics And Comparative Deltas

## Summary

Emit graph-specific evaluation metrics and replayable report artifacts so graph
and linear planner tradeoffs remain inspectable in regression review.

## Acceptance Criteria

- [x] [SRS-02/AC-01] Evaluation reports expose graph-specific metrics such as frontier expansion cost, merge or prune counts, or branch efficiency. <!-- verify: manual, SRS-02:start:end -->
- [x] [SRS-03/AC-02] Graph evaluation artifacts remain replayable and suitable for regression comparison across graph and linear planner revisions. <!-- verify: manual, SRS-03:start:end -->
