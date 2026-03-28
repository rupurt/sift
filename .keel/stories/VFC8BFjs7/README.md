---
# system-managed
id: VFC8BFjs7
status: done
created_at: 2026-03-28T13:09:21
updated_at: 2026-03-28T14:23:13
# authored
title: Add Planner Decisions And Stop Reasons
type: feat
operator-signal:
scope: VFC7H4QFy/VFC7MN6fR
index: 2
started_at: 2026-03-28T14:20:10
submitted_at: 2026-03-28T14:23:07
completed_at: 2026-03-28T14:23:13
---

# Add Planner Decisions And Stop Reasons

## Summary

Introduce replayable planner decision and stop-reason records so autonomous
continuation and termination can be inspected without relying on runtime logs
or implicit controller behavior, and formalize planner strategy selection in
the same contract layer.

## Acceptance Criteria

- [x] [SRS-02/AC-01] Planner decision and stop-reason records exist and can be emitted as replayable trace data. <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-03/AC-02] Planner strategy selection exists as explicit contract data and can represent both heuristic and model-driven policy choices. <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-01/AC-03] The planner contract remains linear-first while carrying stable identifiers or reason codes that can extend toward future branching search. <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-3.log-->
