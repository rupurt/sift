---
# system-managed
id: VFnGb7HrI
status: backlog
created_at: 2026-04-03T21:34:51
updated_at: 2026-04-03T21:41:39
# authored
title: Add Frontier Ledger Rolling Sector Statistics
type: feat
operator-signal:
scope: VFnGRPtQQ/VFnGWurCd
index: 1
---

# Add Frontier Ledger Rolling Sector Statistics

## Summary

Implement the frontier ledger and rolling sector statistics that direct-search coverage signaling will derive from during startup, rebuild, and resume flows.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Frontier ledger state records rolling sector counts, reuse counts, dirty-sector counts, and active rebuild metadata derived from sector and breadcrumb state. <!-- verify: test, SRS-01:start:end -->
- [ ] [SRS-01/AC-02] Frontier ledger updates rolling sector statistics as clean sectors mount, dirty sectors rebuild, and breadcrumb resume state changes. <!-- verify: test, SRS-01:start:end -->
- [ ] [SRS-NFR-01/AC-03] Frontier statistics derive from the existing sector and breadcrumb authorities instead of introducing a second file-state tracker. <!-- verify: manual, SRS-NFR-01:start:end -->
