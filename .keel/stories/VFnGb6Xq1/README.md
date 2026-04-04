---
# system-managed
id: VFnGb6Xq1
status: backlog
created_at: 2026-04-03T21:34:51
updated_at: 2026-04-03T21:41:37
# authored
title: Resume Interrupted Sector Rebuilds On Startup
type: feat
operator-signal:
scope: VFnGRPtQQ/VFnGWuLCk
index: 2
---

# Resume Interrupted Sector Rebuilds On Startup

## Summary

Resume interrupted dirty-sector rebuilds from breadcrumb state on startup while preserving immediate clean-sector mounts and safe corrupt-journal fallback behavior.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] Startup resumes interrupted dirty-sector rebuilds from breadcrumb state while preserving clean-sector mounts. <!-- verify: test, SRS-03:start:end -->
- [ ] [SRS-04/AC-02] Stale or corrupt breadcrumb state is discarded safely without invalidating clean-sector reuse claims. <!-- verify: test, SRS-04:start:end -->
- [ ] [SRS-NFR-01/AC-03] Runtime telemetry or progress counters expose breadcrumb resume and recovery state to operators and embedders. <!-- verify: test, SRS-NFR-01:start:end -->
