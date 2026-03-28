---
# system-managed
id: VFCWVqAzC
status: done
created_at: 2026-03-28T14:46:00
updated_at: 2026-03-28T15:21:38
# authored
title: Add Heuristic Planner Stop Heuristics
type: feat
operator-signal:
scope: VFC7H4QFx/VFCW6PVzz
index: 2
started_at: 2026-03-28T15:18:12
submitted_at: 2026-03-28T15:21:36
completed_at: 2026-03-28T15:21:38
---

# Add Heuristic Planner Stop Heuristics

## Summary

Add explicit heuristic stop conditions so the built-in planner can terminate
bounded linear episodes with replayable reasons instead of relying on implicit
empty-loop behavior.

## Acceptance Criteria

- [x] [SRS-03/AC-01] The heuristic planner emits explicit stop reasons when the step limit is reached or when no productive next query remains. <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-02] Repeated runs over the same request and retained evidence produce deterministic planner decisions and stop outcomes. <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-02/AC-03] The heuristic baseline remains model-free and local-first. <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->
