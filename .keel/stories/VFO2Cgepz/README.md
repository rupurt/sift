---
# system-managed
id: VFO2Cgepz
status: backlog
created_at: 2026-03-30T14:00:53
updated_at: 2026-03-30T14:03:42
# authored
title: Add Progress Callback Parameter To Search Autonomous
type: feat
operator-signal:
scope: VFO1icY5Z/VFO1uSaNE
index: 2
---

# Add Progress Callback Parameter To Search Autonomous

## Summary

Add an optional progress callback parameter to search_autonomous and search_autonomous_with. Existing callers must compile without changes. The callback is generic `impl Fn(&SearchProgress)` for zero-cost monomorphization when unused.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] search_autonomous accepts an optional progress callback parameter <!-- verify: grep, SRS-03 -->
- [ ] [SRS-04/AC-02] search_autonomous_with accepts an optional progress callback parameter <!-- verify: grep, SRS-04 -->
- [ ] [SRS-NFR-02/AC-03] All existing tests pass without modification <!-- verify: cargo nextest run, SRS-NFR-02 -->
- [ ] [SRS-NFR-03/AC-04] No async runtime dependency is introduced <!-- verify: grep, SRS-NFR-03 -->
