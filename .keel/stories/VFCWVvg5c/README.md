---
# system-managed
id: VFCWVvg5c
status: backlog
created_at: 2026-03-28T14:46:00
updated_at: 2026-03-28T14:49:53
# authored
title: Document Autonomous Library Usage
type: docs
operator-signal:
scope: VFC7H4pFw/VFCWCm3Mc
index: 2
---

# Document Autonomous Library Usage

## Summary

Document how embedders use the supported autonomous library surface, including
planner strategy selection, trace handling, and the fact that existing
non-autonomous library modes remain supported.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] Library documentation explains autonomous planner configuration, strategy selection, traces, and supported modes. <!-- verify: manual, SRS-02:start:end -->
- [ ] [SRS-03/AC-02] Strategy-aware autonomous response contracts remain available through the supported library surface and are described for embedders. <!-- verify: manual, SRS-03:start:end -->
- [ ] [SRS-NFR-01/AC-02] Documentation and tests make clear that the supported library surface shares the same runtime that future CLI layers will reuse. <!-- verify: manual, SRS-NFR-01:start:end -->
- [ ] [SRS-NFR-02/AC-03] Existing non-autonomous library modes remain documented and supported alongside the autonomous surface. <!-- verify: manual, SRS-NFR-02:start:end -->
