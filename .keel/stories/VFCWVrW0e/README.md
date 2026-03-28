---
# system-managed
id: VFCWVrW0e
status: backlog
created_at: 2026-03-28T14:46:00
updated_at: 2026-03-28T14:49:53
# authored
title: Preserve Additive Autonomous Runtime Behavior
type: feat
operator-signal:
scope: VFC7H4QFx/VFCW85Y1r
index: 2
---

# Preserve Additive Autonomous Runtime Behavior

## Summary

Keep the autonomous runtime additive: it must reuse shared controller
semantics, support planner-state progression, and leave existing single-turn
and planned-controller invocation paths untouched.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] The autonomous runtime can advance or resume from explicit planner state without introducing a parallel execution stack. <!-- verify: manual, SRS-03:start:end -->
- [ ] [SRS-NFR-01/AC-02] `search_turn` and `search_controller` remain intact when autonomous planning is not selected. <!-- verify: manual, SRS-NFR-01:start:end -->
- [ ] [SRS-NFR-02/AC-03] The autonomous runtime reuses shared controller semantics instead of forking retained-evidence behavior. <!-- verify: manual, SRS-NFR-02:start:end -->
