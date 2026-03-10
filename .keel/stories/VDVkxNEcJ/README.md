---
id: VDVkxNEcJ
title: Document The Runnable Embedded Example
type: docs
status: in-progress
created_at: 2026-03-10T16:44:47
updated_at: 2026-03-10T16:51:53
scope: VDVkH5a6M/VDVkORseE
index: 1
started_at: 2026-03-10T16:51:53
---

# Document The Runnable Embedded Example

## Summary

Update repository docs to present `sift-embed` as the canonical runnable
embedding example and show the repo-root commands that build or run it.

## Acceptance Criteria

- [x] [SRS-03/AC-01] Repository documentation identifies the example crate as the canonical runnable embedding reference and shows `sift-embed search "<term>"` usage. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "sift-embed search|canonical runnable embedding reference|examples/sift-embed" README.md examples/sift-embed/README.md', SRS-03:start:end, proof: ac-1.log -->
- [x] [SRS-03/AC-02] Documentation shows the repo-root `just` recipes used to build or run the example consumer. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "just embed-build|just embed-search" README.md examples/sift-embed/README.md', SRS-03:start:end, proof: ac-2.log -->
