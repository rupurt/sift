# VOYAGE REPORT: Version Output Metadata

## Voyage Metadata
- **ID:** VDWk0yH65
- **Epic:** VDWjylxX2
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Stamp CLI Version Output With Git Metadata
- **ID:** VDWk6czfb
- **Status:** done

#### Summary
Add build-time version metadata so `sift --version` reports semver plus git SHA,
distinguishes development from release builds, and documents the release-workflow
contract for stamping the authoritative commit SHA.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] A normal local build reports `sift <semver>-dev (<sha>)`, using a seven-character SHA in the clap `--version` output. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo run -- --version | rg "^sift [0-9]+\\.[0-9]+\\.[0-9]+-dev \\([0-9a-f]{7}\\)$"', SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-02] A release-profile build reports `sift <semver> (<sha>)` without `-dev`, and the build can source the SHA from an explicit environment variable suitable for GitHub Actions. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && SIFT_GIT_SHA=abcdef123456 cargo run --release -- --version | rg "^sift [0-9]+\\.[0-9]+\\.[0-9]+ \\(abcdef1\\)$"', SRS-02:start:end, proof: ac-2.log-->
- [x] [SRS-03/AC-03] The GitHub release workflow and release documentation describe and apply the commit-SHA injection contract for release artifacts. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "SIFT_GIT_SHA|--version|abcdef1|release build|cargo dist build" .github/workflows/release.yml RELEASE.md', SRS-03:start:end, proof: ac-3.log-->
- [x] [SRS-04/AC-04] Shared version-formatting logic is covered by automated tests for release, dev, and `unknown` fallback cases. <!-- verify: cargo test versioning::, SRS-04:start:end, proof: ac-4.log-->

#### Verified Evidence
- [ac-4.log](../../../../stories/VDWk6czfb/EVIDENCE/ac-4.log)
- [ac-1.log](../../../../stories/VDWk6czfb/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/VDWk6czfb/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/VDWk6czfb/EVIDENCE/ac-2.log)


