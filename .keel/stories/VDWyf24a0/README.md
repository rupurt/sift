---
id: VDWyf24a0
title: Cut 0.2.0 Release Candidate
type: feat
status: backlog
created_at: 2026-03-10T21:45:30
updated_at: 2026-03-10T21:46:23
scope: VDWyNSW9x/VDWyUguh1
index: 1
---

# Cut 0.2.0 Release Candidate

## Summary

Update the repository to version `0.2.0`, refresh release-facing
documentation that still embeds stale semver examples, and record proof that
the existing local and tag-based release paths remain healthy.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `Cargo.toml` declares `version = "0.2.0"` for `sift`, and `cargo run -- --version` reports `sift 0.2.0-dev (<sha-or-unknown>)`. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "^version = \"0\\.2\\.0\"$" Cargo.toml && cargo run -- --version | rg "^sift 0\\.2\\.0-dev \\(([0-9a-f]{7}|unknown)\\)$"', SRS-01:start:end -->
- [ ] [SRS-02/AC-02] Release-facing documentation no longer carries stale `0.1.x` example values where the current release cut should say `0.2.0`. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "0\\.2\\.0" RELEASE.md && ! rg -n "0\\.1\\.[0-9]+" RELEASE.md', SRS-02:start:end -->
- [ ] [SRS-03/AC-03] `just check` passes after the release cut. <!-- verify: just check, SRS-03:start:end -->
- [ ] [SRS-04/AC-04] `cargo dist plan --tag v0.2.0` succeeds against the updated repository state. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo dist plan --tag v0.2.0 >/tmp/sift-dist-plan.log && tail -n 20 /tmp/sift-dist-plan.log', SRS-04:start:end -->
- [ ] [SRS-05/AC-05] The release cut remains limited to version-preparation artifacts and does not add unrelated product changes. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && git diff --name-only --cached | rg "^(Cargo.toml|Cargo.lock|RELEASE.md|\\.keel/)"', SRS-05:start:end -->
