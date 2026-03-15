---
id: VDQ9YwPaS
title: Add Homebrew Installer Support
type: feat
status: done
created_at: 2026-03-09T17:55:00
updated_at: 2026-03-09T17:46:06
scope: VDQ8Ll4DX/VDQ9bZoZV
index: 1
started_at: 2026-03-09T17:58:12
submitted_at: 2026-03-09T17:46:05
completed_at: 2026-03-09T17:46:06
---

# Add Homebrew Installer Support

## Summary

Add Homebrew formula generation to the `cargo-dist` release pipeline to allow users to install `sift` via `brew install`.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `Cargo.toml` includes `homebrew` in the `installers` list <!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Homebrew tap repository is configured <!-- verify: manual, SRS-01:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-03] Documentation updated to include Homebrew installation instructions <!-- verify: manual, SRS-02:start:end, proof: ac-3.log -->
