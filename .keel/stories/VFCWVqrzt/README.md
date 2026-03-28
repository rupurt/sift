---
# system-managed
id: VFCWVqrzt
status: done
created_at: 2026-03-28T14:46:00
updated_at: 2026-03-28T15:25:06
# authored
title: Drive Autonomous Episodes Through Built-In Runtime
type: feat
operator-signal:
scope: VFC7H4QFx/VFCW85Y1r
index: 1
started_at: 2026-03-28T15:22:20
submitted_at: 2026-03-28T15:25:04
completed_at: 2026-03-28T15:25:06
---

# Drive Autonomous Episodes Through Built-In Runtime

## Summary

Add the built-in autonomous runtime path that executes planner-generated search
episodes end to end by lowering planner decisions into the existing shared
controller/search substrate.

## Acceptance Criteria

- [x] [SRS-01/AC-01] A built-in autonomous runtime path can execute planner-generated search decisions without requiring external custom planner injection. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-02] Planner-generated search decisions lower into the shared controller/runtime path with retained evidence carryover between steps. <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
