---
id: VDQAq22Tq
title: Provide Static Linux Executable
type: feat
status: done
created_at: 2026-03-09T18:05:00
updated_at: 2026-03-09T17:51:04
scope: VDQ8Ll4DX/VDQAsfNUx
index: 1
started_at: 2026-03-09T18:13:12
submitted_at: 2026-03-09T17:51:04
completed_at: 2026-03-09T17:51:04
---

# Provide Static Linux Executable

## Summary

Configure the release pipeline to produce a fully static Linux executable using the `musl` target.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `Cargo.toml` includes `x86_64-unknown-linux-musl` target <!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Release workflow includes `x86_64-unknown-linux-musl` job <!-- verify: manual, SRS-01:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-03] `RELEASE.md` updated to reflect static binary availability <!-- verify: manual, SRS-02:start:end, proof: ac-3.log -->
