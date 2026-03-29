---
# system-managed
id: VFDQ2TUmP
status: done
created_at: 2026-03-28T18:26:33
updated_at: 2026-03-28T18:37:27
# authored
title: Execute Graph Episodes Through Bounded Frontier Runtime
type: feat
operator-signal:
scope: VFD8NgvJl/VFD8TRPTh
index: 1
started_at: 2026-03-28T18:30:07
submitted_at: 2026-03-28T18:33:27
completed_at: 2026-03-28T18:37:27
---

# Execute Graph Episodes Through Bounded Frontier Runtime

## Summary

Execute graph episodes over an explicit bounded frontier through the shared
retrieval substrate instead of treating graph runs as a second search stack.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Graph episodes execute over a bounded frontier while reusing the shared retrieval and controller substrate for branch retrieval turns. <!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-NFR-01/AC-02] Frontier execution remains bounded by explicit episode limits and branch limits. <!-- verify: manual, SRS-NFR-01:start:end -->
