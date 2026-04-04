---
# system-managed
id: VFnGb6hpt
status: done
created_at: 2026-04-03T21:34:51
updated_at: 2026-04-03T22:35:13
# authored
title: Route Controller And Autonomous Search Through Sector Preparation
type: feat
operator-signal:
scope: VFnGRPtQQ/VFnGWulCe
index: 1
started_at: 2026-04-03T22:31:09
submitted_at: 2026-04-03T22:35:07
completed_at: 2026-04-03T22:35:13
---

# Route Controller And Autonomous Search Through Sector Preparation

## Summary

Route controller and autonomous startup through the shared sector-aware preparation seam so those runtime surfaces reuse the same cache, coverage, and progress contracts as direct search.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Controller and autonomous search startup uses the shared sector-aware preparation path instead of duplicate whole-corpus rebuild logic. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test search_controller_reuses_clean_sectors_on_warm_restart && cargo test autonomous_search_reuses_clean_sectors_on_warm_restart', SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-02] Controller, autonomous, CLI, and library runtime surfaces expose the same sector reuse and coverage metrics contract as direct search. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test search_controller_reuses_clean_sectors_on_warm_restart && cargo test autonomous_search_reuses_clean_sectors_on_warm_restart && cargo test progress_renderer_includes_indexing_cache_metrics', SRS-02:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-01/AC-03] Runtime adoption preserves one preparation authority and one cache substrate instead of branching into per-surface startup implementations. <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-3.log-->
