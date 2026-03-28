---
# system-managed
id: VFCWVv04r
status: done
created_at: 2026-03-28T14:46:00
updated_at: 2026-03-28T15:46:18
# authored
title: Expose Supported Autonomous Library Entry Point
type: feat
operator-signal:
scope: VFC7H4pFw/VFCWCm3Mc
index: 1
started_at: 2026-03-28T15:44:15
submitted_at: 2026-03-28T15:46:15
completed_at: 2026-03-28T15:46:18
---

# Expose Supported Autonomous Library Entry Point

## Summary

Promote the built-in autonomous runtime to a supported crate-root entry point
that invokes built-in planner strategies without forcing embedders to supply a
custom planner implementation.

## Acceptance Criteria

- [x] [SRS-01/AC-01] A supported crate-root autonomous entry point invokes built-in planner strategies without custom planner injection. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
