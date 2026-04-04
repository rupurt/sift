---
# system-managed
id: VFnCsXrSB
status: done
created_at: 2026-04-03T21:20:05
updated_at: 2026-04-03T21:30:10
# authored
title: Support Frontier Converging And Sealed Search Coverage
type: feat
operator-signal:
scope: VFnCKDDhj/VFnCTN04l
index: 3
started_at: 2026-04-03T21:29:54
submitted_at: 2026-04-03T21:30:07
completed_at: 2026-04-03T21:30:10
---

# Support Frontier Converging And Sealed Search Coverage

## Summary

Define the partial-coverage search semantics and scoring strategy that make frontier hunting useful before a fully sealed index exists.

## Acceptance Criteria

- [x] [SRS-04/AC-01] The design defines a `frontier` search mode that can search ready sectors before a fully sealed corpus index exists. <!-- verify: manual, SRS-04:start:end -->
- [x] [SRS-05/AC-02] The design defines how frontier and converging search obtain usable ranking statistics without waiting for a prior complete global snapshot. <!-- verify: manual, SRS-05:start:end -->
- [x] [SRS-06/AC-03] The design defines explicit coverage-state signaling for `frontier`, `converging`, and `sealed` search through progress or API surfaces. <!-- verify: manual, SRS-06:start:end -->
