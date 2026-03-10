---
id: VDVSF0yAX
title: Introduce Canonical Embedded Search Facade
type: feat
status: backlog
created_at: 2026-03-10T15:30:27
updated_at: 2026-03-10T15:31:02
scope: VDVQurZER/VDVRkNjgH
index: 1
---

# Introduce Canonical Embedded Search Facade

## Summary

Create the first supported high-level library entrypoint for running `sift`
search from another Rust project so embedders no longer need to compose broad
internal modules manually.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] The library exposes one canonical embedded search facade with a high-level request/response path suitable for external Rust callers. <!-- verify: cargo test, SRS-01:start:end -->
- [ ] [SRS-01/AC-02] Integration coverage exercises the supported facade directly rather than reaching through multiple internal modules. <!-- verify: cargo test, SRS-01:start:end -->
