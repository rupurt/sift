---
# system-managed
id: VFnGb64p8
status: done
created_at: 2026-04-03T21:34:51
updated_at: 2026-04-03T22:12:42
# authored
title: Load Clean Sector Shards For Direct Search Startup
type: feat
operator-signal:
scope: VFnGRPtQQ/VFnGWuRCh
index: 1
started_at: 2026-04-03T22:04:10
submitted_at: 2026-04-03T22:12:39
completed_at: 2026-04-03T22:12:42
---

# Load Clean Sector Shards For Direct Search Startup

## Summary

Implement the direct-search startup path that loads clean sectors first, rebuilds only dirty sectors, and proves warm restart reuse over unchanged corpora.

## Acceptance Criteria

- [x] [SRS-02/AC-01] Direct-search startup loads clean sectors from persisted state before rebuilding dirty sectors. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test direct_search_reuses_clean_sectors_on_warm_restart', SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-03/AC-02] Dirty sectors rebuild in isolation and persist refreshed sector-local lexical shards without invalidating unrelated clean sectors. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test direct_search_rebuilds_only_the_dirty_sector', SRS-03:start:end, proof: ac-2.log-->
- [x] [SRS-04/AC-03] Warm direct-search restart over an unchanged corpus can begin from mounted clean sectors without a whole-corpus reparse before first useful results. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test direct_search_reuses_clean_sectors_on_warm_restart', SRS-04:start:end, proof: ac-3.log-->
- [x] [SRS-NFR-02/AC-04] The slice delivers direct-search reuse without requiring controller or autonomous runtime changes. <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-4.log-->
