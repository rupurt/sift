---
id: VDWk6czfb
title: Stamp CLI Version Output With Git Metadata
type: feat
status: backlog
created_at: 2026-03-10T20:47:41
updated_at: 2026-03-10T20:50:23
scope: VDWjylxX2/VDWk0yH65
index: 1
---

# Stamp CLI Version Output With Git Metadata

## Summary

Add build-time version metadata so `sift --version` reports semver plus git SHA,
distinguishes development from release builds, and documents the release-workflow
contract for stamping the authoritative commit SHA.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] A normal local build reports `sift <semver>-dev (<sha>)`, using a seven-character SHA in the clap `--version` output. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo run -- --version | rg "^sift [0-9]+\\.[0-9]+\\.[0-9]+-dev \\([0-9a-f]{7}\\)$"', SRS-01:start:end -->
- [ ] [SRS-02/AC-02] A release-profile build reports `sift <semver> (<sha>)` without `-dev`, and the build can source the SHA from an explicit environment variable suitable for GitHub Actions. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && SIFT_GIT_SHA=abcdef123456 cargo run --release -- --version | rg "^sift [0-9]+\\.[0-9]+\\.[0-9]+ \\(abcdef1\\)$"', SRS-02:start:end -->
- [ ] [SRS-03/AC-03] The GitHub release workflow and release documentation describe and apply the commit-SHA injection contract for release artifacts. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "SIFT_GIT_SHA|--version|abcdef1|release build|cargo dist build" .github/workflows/release.yml RELEASE.md', SRS-03:start:end -->
- [ ] [SRS-04/AC-04] Shared version-formatting logic is covered by automated tests for release, dev, and `unknown` fallback cases. <!-- verify: cargo test versioning::, SRS-04:start:end -->
