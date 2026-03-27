---
id: VF1VuFbgz
title: Expose Stable Agentic Entry Points In The Public Facade
type: feat
status: done
created_at: 2026-03-26T17:34:36
updated_at: 2026-03-26T19:01:10
operator-signal: 
scope: VF1VhxmqI/VF1Vt0WCT
index: 2
started_at: 2026-03-26T19:00:55
submitted_at: 2026-03-26T19:01:07
completed_at: 2026-03-26T19:01:10
---

# Expose Stable Agentic Entry Points In The Public Facade

## Summary

Promote the new turn and emission contracts through a supported public API so embedders do not need to depend on `sift::internal`.

## Acceptance Criteria

- [x] [SRS-04/AC-01] A supported facade or crate-root entry point exposes the new contracts. <!-- verify: manual, SRS-04:start:end -->
- [x] [SRS-NFR-01/AC-02] Existing single-turn hybrid callers remain supported during the cutover. <!-- verify: manual, SRS-NFR-01:start:end -->
