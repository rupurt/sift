---
# system-managed
id: VFCWVwR6R
status: done
created_at: 2026-03-28T14:46:00
updated_at: 2026-03-28T15:50:36
# authored
title: Add Autonomous Planner Flag To Search CLI
type: feat
operator-signal:
scope: VFC7H4pFw/VFCWG2tZd
index: 1
started_at: 2026-03-28T15:46:38
submitted_at: 2026-03-28T15:50:32
completed_at: 2026-03-28T15:50:36
---

# Add Autonomous Planner Flag To Search CLI

## Summary

Add planner-driven search to the shipped executable through `sift search
--agent`, reusing the shared autonomous runtime instead of inventing a CLI-only
planner path.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `sift search --agent` invokes the shared autonomous runtime from the executable. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-02] Agent-mode CLI output and JSON expose planner strategy and autonomous trace metadata suitable for inspection and downstream tooling. <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
