---
# system-managed
id: VFnGb6ypo
status: backlog
created_at: 2026-04-03T21:34:51
updated_at: 2026-04-03T21:41:39
# authored
title: Surface Frontier Coverage Through Progress And Responses
type: feat
operator-signal:
scope: VFnGRPtQQ/VFnGWurCd
index: 2
---

# Surface Frontier Coverage Through Progress And Responses

## Summary

Extend direct-search progress and result surfaces to expose `frontier`, `converging`, and `sealed` coverage states alongside rolling sector statistics.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] Direct-search preparation computes `frontier`, `converging`, and `sealed` coverage states from mounted, dirty, and resumed sectors and updates them as indexing advances. <!-- verify: test, SRS-02:start:end -->
- [ ] [SRS-03/AC-02] Direct-search progress snapshots and search responses expose coverage mode plus sector statistics so callers can distinguish partial results from sealed coverage. <!-- verify: test, SRS-03:start:end -->
- [ ] [SRS-04/AC-03] Coverage signaling remains conservative during resume, recovery, and dirty-sector rebuilds and never reports `sealed` before all reachable dirty sectors converge. <!-- verify: test, SRS-04:start:end -->
- [ ] [SRS-NFR-02/AC-04] Coverage visibility does not require an extra whole-corpus validation pass before first useful progress or results are surfaced. <!-- verify: command, SRS-NFR-02:start:end -->
