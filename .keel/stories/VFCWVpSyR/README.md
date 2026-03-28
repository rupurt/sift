---
# system-managed
id: VFCWVpSyR
status: done
created_at: 2026-03-28T14:46:00
updated_at: 2026-03-28T15:18:07
# authored
title: Implement Heuristic Planner Query Generation
type: feat
operator-signal:
scope: VFC7H4QFx/VFCW6PVzz
index: 1
started_at: 2026-03-28T15:11:22
submitted_at: 2026-03-28T15:18:04
completed_at: 2026-03-28T15:18:07
---

# Implement Heuristic Planner Query Generation

## Summary

Implement the first built-in heuristic planner policy that can derive initial
and follow-up autonomous search decisions from a root task, retained evidence,
and local context without caller-authored turn lists.

## Acceptance Criteria

- [x] [SRS-01/AC-01] A heuristic planner policy exists and can emit an initial autonomous search decision from a root task plus current local context. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-02] The heuristic planner can derive and deduplicate follow-up search decisions from retained evidence and prior planner output. <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
