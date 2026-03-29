---
# system-managed
id: VFDACChy2
status: backlog
created_at: 2026-03-28T17:23:38
updated_at: 2026-03-28T17:24:08
# authored
title: Surface Explicit Graph Trace Contract Errors
type: feat
operator-signal:
scope: VFD8KR44d/VFD8TQyTg
index: 3
---

# Surface Explicit Graph Trace Contract Errors

## Summary

Surface explicit validation failures for impossible graph traces so runtime and
planner drift are visible as contract errors instead of hidden repair logic.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] Validation failures surface explicit contract errors for impossible graph transitions instead of silently repairing invalid traces. <!-- verify: manual, SRS-03:start:end -->
- [ ] [SRS-NFR-02/AC-02] Invalid graph traces leave the current linear autonomous path untouched when graph mode is not selected. <!-- verify: manual, SRS-NFR-02:start:end -->
