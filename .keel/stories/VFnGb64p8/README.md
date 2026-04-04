---
# system-managed
id: VFnGb64p8
status: backlog
created_at: 2026-04-03T21:34:51
updated_at: 2026-04-03T21:41:35
# authored
title: Load Clean Sector Shards For Direct Search Startup
type: feat
operator-signal:
scope: VFnGRPtQQ/VFnGWuRCh
index: 1
---

# Load Clean Sector Shards For Direct Search Startup

## Summary

Implement the direct-search startup path that loads clean sectors first, rebuilds only dirty sectors, and proves warm restart reuse over unchanged corpora.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] Direct-search startup loads clean sectors from persisted state before rebuilding dirty sectors. <!-- verify: test, SRS-02:start:end -->
- [ ] [SRS-03/AC-02] Dirty sectors rebuild in isolation and persist refreshed sector-local lexical shards without invalidating unrelated clean sectors. <!-- verify: test, SRS-03:start:end -->
- [ ] [SRS-04/AC-03] Warm direct-search restart over an unchanged corpus can begin from mounted clean sectors without a whole-corpus reparse before first useful results. <!-- verify: command, SRS-04:start:end -->
- [ ] [SRS-NFR-02/AC-04] The slice delivers direct-search reuse without requiring controller or autonomous runtime changes. <!-- verify: manual, SRS-NFR-02:start:end -->
