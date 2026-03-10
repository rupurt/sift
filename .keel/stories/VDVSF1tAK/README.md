---
id: VDVSF1tAK
title: Document The Supported Embedded API Boundary
type: docs
status: in-progress
created_at: 2026-03-10T15:30:27
updated_at: 2026-03-10T15:59:24
scope: VDVQurZER/VDVRkNjgH
index: 4
started_at: 2026-03-10T15:59:24
---

# Document The Supported Embedded API Boundary

## Summary

Document which exports and modules are part of the supported embedded API and
trim or mark the rest as internal, then add a concrete guide showing how
another Rust project should actually depend on and call the library.

## Acceptance Criteria

- [x] [SRS-04/AC-01] Repository documentation explains the supported embedded API path and distinguishes it from internal implementation modules. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && rg -n "## Embedded Library|supported library contract|sift::internal" README.md && rg -n "Supported embedded API|Everything under \\[`internal`\\]" src/lib.rs', SRS-04:start:end, proof: ac-1.log-->
- [x] [SRS-05/AC-02] Repository documentation includes a concrete library-usage guide with the supported dependency and a minimal embedding example. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && rg -n "git = \\\"https://github.com/rupurt/sift\\\"|path = \\\"../sift\\\"|Minimal Embedding Example|let response = sift.search|SearchInput::new" README.md', SRS-05:start:end, proof: ac-2.log-->
- [x] [SRS-06/AC-03] The public export surface is intentionally narrower than the current broad module re-export pattern, preserving the single-package rollout without adding an immediate workspace split. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && cargo check --all-targets && rg -n "^pub mod internal" src/lib.rs && ! rg -n "^pub mod (cache|config|dense|eval|extract|hybrid|search|segment|system|vector);" src/lib.rs', SRS-06:start:end, proof: ac-3.log-->
