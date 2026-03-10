---
id: VDVkxNEcJ
title: Document The Runnable Embedded Example
type: docs
status: backlog
created_at: 2026-03-10T16:44:47
updated_at: 2026-03-10T16:45:26
scope: VDVkH5a6M/VDVkORseE
index: 1
---

# Document The Runnable Embedded Example

## Summary

Update repository docs to present `sift-embed` as the canonical runnable
embedding example and show the repo-root commands that build or run it.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] Repository documentation identifies the example crate as the canonical runnable embedding reference and shows `sift-embed search "<term>"` usage. <!-- verify: rg -n 'sift-embed search|canonical runnable embedding reference|examples/sift-embed' README.md examples/sift-embed/README.md, SRS-03:start:end -->
- [ ] [SRS-03/AC-02] Documentation shows the repo-root `just` recipes used to build or run the example consumer. <!-- verify: rg -n 'just embed-build|just embed-search' README.md examples/sift-embed/README.md, SRS-03:start:end -->
