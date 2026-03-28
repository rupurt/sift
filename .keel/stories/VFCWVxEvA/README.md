---
# system-managed
id: VFCWVxEvA
status: backlog
created_at: 2026-03-28T14:46:00
updated_at: 2026-03-28T14:49:53
# authored
title: Preserve Non-Agent Search UX
type: feat
operator-signal:
scope: VFC7H4pFw/VFCWG2tZd
index: 2
---

# Preserve Non-Agent Search UX

## Summary

Preserve the current non-agent search command behavior and surrounding
search/evaluation workflows while the executable gains an explicit agent-mode
surface.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] Existing non-agent `sift search` behavior remains unchanged when `--agent` is not selected. <!-- verify: manual, SRS-03:start:end -->
- [ ] [SRS-NFR-01/AC-02] The CLI agent surface continues to call the same runtime contract as the supported library autonomous path. <!-- verify: manual, SRS-NFR-01:start:end -->
- [ ] [SRS-NFR-02/AC-03] Existing search and evaluation commands do not regress when the agent flag is added. <!-- verify: manual, SRS-NFR-02:start:end -->
