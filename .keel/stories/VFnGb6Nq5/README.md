---
# system-managed
id: VFnGb6Nq5
status: backlog
created_at: 2026-04-03T21:34:51
updated_at: 2026-04-03T21:41:37
# authored
title: Persist Breadcrumb Journals During Sector Rebuilds
type: feat
operator-signal:
scope: VFnGRPtQQ/VFnGWuLCk
index: 1
---

# Persist Breadcrumb Journals During Sector Rebuilds

## Summary

Persist breadcrumb journals during dirty-sector rebuild work so fresh processes have a trusted resumability record without disturbing clean-sector reuse.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Breadcrumb journal records persist run identity, completed sectors, dirty sectors, active sector state, and resume cursor metadata. <!-- verify: test, SRS-01:start:end -->
- [ ] [SRS-02/AC-02] Dirty-sector rebuild work checkpoints breadcrumb progress during sector processing. <!-- verify: test, SRS-02:start:end -->
- [ ] [SRS-NFR-02/AC-03] Breadcrumb persistence remains fully local-first and does not require a daemon or external coordinator. <!-- verify: manual, SRS-NFR-02:start:end -->
