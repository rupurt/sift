---
id: VF60Mx7wx
title: Refactor File And Project Doc Ingestion Behind Local Acquisition Adapters
type: feat
status: done
created_at: 2026-03-27T12:00:41
updated_at: 2026-03-27T13:21:36
operator-signal: 
scope: VF53su1Xu/VF60I100k
index: 1
started_at: 2026-03-27T13:19:19
submitted_at: 2026-03-27T13:21:27
completed_at: 2026-03-27T13:21:36
---

# Refactor File And Project Doc Ingestion Behind Local Acquisition Adapters

## Summary

Move local file and project-document ingestion behind the new acquisition
adapter seam so those sources stop bypassing the shared context substrate.

## Acceptance Criteria

- [x] [SRS-01/AC-01] File-backed content and project docs are acquired through explicit local acquisition adapters. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-02] The refactor preserves the local-first, zero-daemon execution model. <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-2.log-->
