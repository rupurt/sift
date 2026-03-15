# VOYAGE REPORT: Prepare 0.2.0 Release

## Voyage Metadata
- **ID:** VDWyUguh1
- **Epic:** VDWyNSW9x
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Cut 0.2.0 Release Candidate
- **ID:** VDWyf24a0
- **Status:** done

#### Summary
Update the repository to version `0.2.0`, refresh release-facing
documentation that still embeds stale semver examples, and record proof that
the existing local and tag-based release paths remain healthy.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `Cargo.toml` declares `version = "0.2.0"` for `sift`, and `cargo run -- --version` reports `sift 0.2.0-dev (<sha-or-unknown>)`. <!-- verify: nix develop -c sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "^version = \"0\\.2\\.0\"$" Cargo.toml && cargo run -- --version | rg "^sift 0\\.2\\.0-dev \\(([0-9a-f]{7}|unknown)\\)$"', SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-02] Release-facing documentation no longer carries stale `0.1.x` example values where the current release cut should say `0.2.0`. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "0\\.2\\.0" RELEASE.md && ! rg -n "0\\.1\\.[0-9]+" RELEASE.md', SRS-02:start:end, proof: ac-2.log-->
- [x] [SRS-03/AC-03] `just check` passes after the release cut. <!-- verify: nix develop -c sh -lc 'cd "$(git rev-parse --show-toplevel)" && just check', SRS-03:start:end, proof: ac-3.log-->
- [x] [SRS-04/AC-04] `cargo dist plan --tag v0.2.0` succeeds against the updated repository state. <!-- verify: nix develop -c sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo dist plan --tag v0.2.0', SRS-04:start:end, proof: ac-4.log-->
- [x] [SRS-05/AC-05] The release cut remains limited to version-preparation artifacts and does not add unrelated product changes. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && git show --name-only --pretty=format: HEAD | rg -v "^$" | tee /tmp/sift-release-files.log | awk "/^(Cargo.toml|Cargo.lock|RELEASE.md|examples\\/sift-embed\\/Cargo.toml|examples\\/sift-embed\\/Cargo.lock|\\.keel\\/)/ { next } { bad = 1 } END { exit bad }"', SRS-05:start:end, proof: ac-5.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VDWyf24a0/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VDWyf24a0/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VDWyf24a0/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VDWyf24a0/EVIDENCE/ac-4.log)
- [ac-5.log](../../../../stories/VDWyf24a0/EVIDENCE/ac-5.log)


