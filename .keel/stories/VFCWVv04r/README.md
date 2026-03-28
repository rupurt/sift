---
# system-managed
id: VFCWVv04r
status: backlog
created_at: 2026-03-28T14:46:00
updated_at: 2026-03-28T14:49:53
# authored
title: Expose Supported Autonomous Library Entry Point
type: feat
operator-signal:
scope: VFC7H4pFw/VFCWCm3Mc
index: 1
---

# Expose Supported Autonomous Library Entry Point

## Summary

Promote the built-in autonomous runtime to a supported crate-root entry point
that invokes built-in planner strategies without forcing embedders to supply a
custom planner implementation.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] A supported crate-root autonomous entry point invokes built-in planner strategies without custom planner injection. <!-- verify: manual, SRS-01:start:end -->
