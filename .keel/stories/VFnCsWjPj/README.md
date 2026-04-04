---
# system-managed
id: VFnCsWjPj
status: done
created_at: 2026-04-03T21:20:05
updated_at: 2026-04-03T21:30:04
# authored
title: Persist Breadcrumb Journals For Resumable Indexing
type: feat
operator-signal:
scope: VFnCKDDhj/VFnCTN04l
index: 2
started_at: 2026-04-03T21:29:54
submitted_at: 2026-04-03T21:30:01
completed_at: 2026-04-03T21:30:04
---

# Persist Breadcrumb Journals For Resumable Indexing

## Summary

Define breadcrumb persistence that records in-progress indexing work so interrupted sector builds can resume across process restarts.

## Acceptance Criteria

- [x] [SRS-03/AC-01] The design defines a persisted `BreadcrumbJournal` that records completed sectors, active sector work, and resumable indexing checkpoints. <!-- verify: manual, SRS-03:start:end -->
- [x] [SRS-NFR-02/AC-02] The design explains how breadcrumb resume and invalid breadcrumb recovery are surfaced to operators and embedders. <!-- verify: manual, SRS-NFR-02:start:end -->
