---
# system-managed
id: VFnGb6Nq5
status: done
created_at: 2026-04-03T21:34:51
updated_at: 2026-04-03T22:21:35
# authored
title: Persist Breadcrumb Journals During Sector Rebuilds
type: feat
operator-signal:
scope: VFnGRPtQQ/VFnGWuLCk
index: 1
started_at: 2026-04-03T22:17:37
submitted_at: 2026-04-03T22:21:32
completed_at: 2026-04-03T22:21:35
---

# Persist Breadcrumb Journals During Sector Rebuilds

## Summary

Persist breadcrumb journals during dirty-sector rebuild work so fresh processes have a trusted resumability record without disturbing clean-sector reuse.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Breadcrumb journal records persist run identity, completed sectors, dirty sectors, active sector state, and resume cursor metadata. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test breadcrumb', SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-02] Dirty-sector rebuild work checkpoints breadcrumb progress during sector processing. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test persists_breadcrumb_journal_during_dirty_sector_rebuild', SRS-02:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-02/AC-03] Breadcrumb persistence remains fully local-first and does not require a daemon or external coordinator. <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->
