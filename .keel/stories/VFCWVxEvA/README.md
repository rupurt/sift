---
# system-managed
id: VFCWVxEvA
status: done
created_at: 2026-03-28T14:46:00
updated_at: 2026-03-28T15:50:36
# authored
title: Preserve Non-Agent Search UX
type: feat
operator-signal:
scope: VFC7H4pFw/VFCWG2tZd
index: 2
started_at: 2026-03-28T15:47:55
submitted_at: 2026-03-28T15:50:33
completed_at: 2026-03-28T15:50:36
---

# Preserve Non-Agent Search UX

## Summary

Preserve the current non-agent search command behavior and surrounding
search/evaluation workflows while the executable gains an explicit agent-mode
surface.

## Acceptance Criteria

- [x] [SRS-03/AC-01] Existing non-agent `sift search` behavior remains unchanged when `--agent` is not selected. <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-02] The CLI agent surface continues to call the same runtime contract as the supported library autonomous path. <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-02/AC-03] Existing search and evaluation commands do not regress when the agent flag is added. <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->
