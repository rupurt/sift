---
# system-managed
id: VFC8BFDri
status: backlog
created_at: 2026-03-28T13:09:21
updated_at: 2026-03-28T13:09:50
# authored
title: Introduce Autonomous Planner Contracts
type: feat
operator-signal:
scope: VFC7H4QFy/VFC7MN6fR
index: 1
---

# Introduce Autonomous Planner Contracts

## Summary

Add the first supported autonomous-planning request, response, and state
records so planner-driven search can begin from a root task instead of only
replaying caller-supplied turns.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Supported autonomous planner request, response, and state records exist for root task, retained evidence, planner strategy, current linear step, and completion status. <!-- verify: manual, SRS-01:start:end -->
