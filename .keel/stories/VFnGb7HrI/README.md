---
# system-managed
id: VFnGb7HrI
status: done
created_at: 2026-04-03T21:34:51
updated_at: 2026-04-03T22:46:24
# authored
title: Add Frontier Ledger Rolling Sector Statistics
type: feat
operator-signal:
scope: VFnGRPtQQ/VFnGWurCd
index: 1
started_at: 2026-04-03T22:40:12
submitted_at: 2026-04-03T22:46:19
completed_at: 2026-04-03T22:46:24
---

# Add Frontier Ledger Rolling Sector Statistics

## Summary

Implement the frontier ledger and rolling sector statistics that direct-search coverage signaling will derive from during startup, rebuild, and resume flows.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Frontier ledger state records rolling sector counts, reuse counts, dirty-sector counts, and active rebuild metadata derived from sector and breadcrumb state. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test frontier_ledger && cargo test frontier_snapshot_reflects_warm_sector_reuse', SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-01/AC-02] Frontier ledger updates rolling sector statistics as clean sectors mount, dirty sectors rebuild, and breadcrumb resume state changes. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test resumes_interrupted_dirty_sector_rebuilds_from_breadcrumb_state && cargo test frontier_snapshot_reflects_warm_sector_reuse', SRS-01:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-01/AC-03] Frontier statistics derive from the existing sector and breadcrumb authorities instead of introducing a second file-state tracker. <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-3.log-->
