---
# system-managed
id: VFnGb6hpt
status: backlog
created_at: 2026-04-03T21:34:51
updated_at: 2026-04-03T21:41:42
# authored
title: Route Controller And Autonomous Search Through Sector Preparation
type: feat
operator-signal:
scope: VFnGRPtQQ/VFnGWulCe
index: 1
---

# Route Controller And Autonomous Search Through Sector Preparation

## Summary

Route controller and autonomous startup through the shared sector-aware preparation seam so those runtime surfaces reuse the same cache, coverage, and progress contracts as direct search.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Controller and autonomous search startup uses the shared sector-aware preparation path instead of duplicate whole-corpus rebuild logic. <!-- verify: test, SRS-01:start:end -->
- [ ] [SRS-02/AC-02] Controller, autonomous, CLI, and library runtime surfaces expose the same sector reuse and coverage metrics contract as direct search. <!-- verify: test, SRS-02:start:end -->
- [ ] [SRS-NFR-01/AC-03] Runtime adoption preserves one preparation authority and one cache substrate instead of branching into per-surface startup implementations. <!-- verify: manual, SRS-NFR-01:start:end -->
