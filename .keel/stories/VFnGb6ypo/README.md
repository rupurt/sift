---
# system-managed
id: VFnGb6ypo
status: done
created_at: 2026-04-03T21:34:51
updated_at: 2026-04-03T22:56:51
# authored
title: Surface Frontier Coverage Through Progress And Responses
type: feat
operator-signal:
scope: VFnGRPtQQ/VFnGWurCd
index: 2
started_at: 2026-04-03T22:46:49
completed_at: 2026-04-03T22:56:51
---

# Surface Frontier Coverage Through Progress And Responses

## Summary

Extend direct-search progress and result surfaces to expose `frontier`, `converging`, and `sealed` coverage states alongside rolling sector statistics.

## Acceptance Criteria

- [x] [SRS-02/AC-01] Direct-search preparation computes `frontier`, `converging`, and `sealed` coverage states from mounted, dirty, and resumed sectors and updates them as indexing advances. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test direct_search_progress_surfaces_frontier_then_converging_then_sealed && cargo test coverage_snapshot_reports_frontier_before_dirty_rebuild_starts && cargo test coverage_snapshot_reports_converging_during_active_or_resumed_rebuilds && cargo test coverage_snapshot_reports_sealed_once_dirty_work_converges', SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-03/AC-02] Direct-search progress snapshots and search responses expose coverage mode plus sector statistics so callers can distinguish partial results from sealed coverage. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test text_output_includes_coverage_summary && cargo test json_output_contains_result_fields && cargo test progress_renderer_includes_indexing_cache_metrics && cargo test direct_search_with_progress_emits_indexing_and_ranking', SRS-03:start:end, proof: ac-2.log-->
- [x] [SRS-04/AC-03] Coverage signaling remains conservative during resume, recovery, and dirty-sector rebuilds and never reports `sealed` before all reachable dirty sectors converge. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test resumes_interrupted_dirty_sector_rebuilds_from_breadcrumb_state && cargo test direct_search_progress_surfaces_frontier_then_converging_then_sealed', SRS-04:start:end, proof: ac-3.log-->
- [x] [SRS-NFR-02/AC-04] Coverage visibility does not require an extra whole-corpus validation pass before first useful progress or results are surfaced. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test direct_search_progress_surfaces_frontier_then_converging_then_sealed && cargo test progress_renderer_includes_indexing_cache_metrics', SRS-NFR-02:start:end, proof: ac-4.log-->
