---
# system-managed
id: VFnGb6Xq1
status: done
created_at: 2026-04-03T21:34:51
updated_at: 2026-04-03T22:29:52
# authored
title: Resume Interrupted Sector Rebuilds On Startup
type: feat
operator-signal:
scope: VFnGRPtQQ/VFnGWuLCk
index: 2
started_at: 2026-04-03T22:21:59
completed_at: 2026-04-03T22:29:52
---

# Resume Interrupted Sector Rebuilds On Startup

## Summary

Resume interrupted dirty-sector rebuilds from breadcrumb state on startup while preserving immediate clean-sector mounts and safe corrupt-journal fallback behavior.

## Acceptance Criteria

- [x] [SRS-03/AC-01] Startup resumes interrupted dirty-sector rebuilds from breadcrumb state while preserving clean-sector mounts. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test resumes_interrupted_dirty_sector_rebuilds_from_breadcrumb_state', SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-04/AC-02] Stale or corrupt breadcrumb state is discarded safely without invalidating clean-sector reuse claims. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test discards_stale_breadcrumb_journal_before_restart && cargo test discards_corrupt_breadcrumb_journal_before_restart', SRS-04:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-01/AC-03] Runtime telemetry or progress counters expose breadcrumb resume and recovery state to operators and embedders. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test progress_renderer_includes_indexing_cache_metrics', SRS-NFR-01:start:end, proof: ac-3.log-->
