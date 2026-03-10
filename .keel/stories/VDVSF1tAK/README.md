---
id: VDVSF1tAK
title: Document The Supported Embedded API Boundary
type: docs
status: backlog
created_at: 2026-03-10T15:30:27
updated_at: 2026-03-10T15:31:02
scope: VDVQurZER/VDVRkNjgH
index: 4
---

# Document The Supported Embedded API Boundary

## Summary

Document which exports and modules are part of the supported embedded API and
trim or mark the rest as internal, then add a concrete guide showing how
another Rust project should actually depend on and call the library.

## Acceptance Criteria

- [ ] [SRS-04/AC-01] Repository documentation explains the supported embedded API path and distinguishes it from internal implementation modules. <!-- verify: manual, SRS-04:start:end -->
- [ ] [SRS-05/AC-02] Repository documentation includes a concrete library-usage guide with the supported dependency and a minimal embedding example. <!-- verify: manual, SRS-05:start:end -->
- [ ] [SRS-06/AC-03] The public export surface is intentionally narrower than the current broad module re-export pattern, preserving the single-package rollout without adding an immediate workspace split. <!-- verify: manual, SRS-06:start:end -->
