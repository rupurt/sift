# Stamped Version Metadata - Product Requirements

## Problem Statement

Users need `sift --version` to identify both the package version and the exact build commit so local builds, CI artifacts, and released binaries can be distinguished without guesswork.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make local and repository builds self-identifying during development. | `sift --version` reports `<semver>-dev (<sha>)` for non-release builds and still renders when git metadata is unavailable. | One canonical version string contract implemented in the binary |
| GOAL-02 | Make release artifacts self-identifying without manual stamping work. | Release/dist builds report `<semver> (<sha>)` and GitHub release jobs inject the exact commit SHA into the build. | Current release workflow stamps commit metadata automatically |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| CLI User | A developer or operator running a local or downloaded `sift` binary. | Immediate confidence about which build they are running. |
| Release Maintainer | A contributor cutting or validating GitHub releases. | Release artifacts that encode both the release semver and exact commit automatically. |

## Scope

### In Scope

- [SCOPE-01] Add build-time version metadata generation for the `sift` binary.
- [SCOPE-02] Distinguish development vs release/dist builds in the reported version string.
- [SCOPE-03] Wire the GitHub release workflow to pass the authoritative commit SHA into artifact builds.
- [SCOPE-04] Document the version metadata contract in release-facing documentation.

### Out of Scope

- [SCOPE-05] Changing the package semver source of truth away from `Cargo.toml`.
- [SCOPE-06] Adding a separate runtime subcommand for build metadata beyond the existing clap `--version` surface.
- [SCOPE-07] Stamping `sift-embed` or other auxiliary binaries in this slice.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The `sift` CLI must render `sift <semver>-dev (<sha>)` for non-release builds and `sift <semver>-dev (unknown)` when git metadata cannot be resolved. | GOAL-01 | must | Local and source-only builds need unambiguous provenance without requiring git at runtime. |
| FR-02 | Release/dist builds must render `sift <semver> (<sha>)` without the `-dev` suffix. | GOAL-02 | must | Official artifacts should present the release version, not a development marker. |
| FR-03 | The GitHub release workflow and release documentation must define how commit SHA metadata is provided to release builds. | GOAL-02 | must | The release path needs a deterministic, documented contract instead of implicit behavior. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Version metadata must be resolved at build time without introducing runtime git dependencies, and fallback behavior must remain deterministic when metadata is unavailable. | GOAL-01, GOAL-02 | must | `--version` should stay fast, portable, and reliable across source and release builds. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| CLI version output | Targeted command proofs for debug and release/dist builds plus unit tests for shared formatting logic | Story proof logs and test results |
| Release workflow | Workflow source inspection plus release-profile command proof using injected SHA env vars | Story proof logs tied to `.github/workflows/release.yml` |
| Fallback behavior | Automated tests for unknown-sha formatting paths | Story proof logs and test results |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Cargo build profiles are sufficient to distinguish normal dev/test builds from release/dist artifact builds. | The version suffix could be wrong for some build paths and would require an explicit environment contract. | Validate during implementation against debug, release, and cargo-dist-oriented paths. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Some source distributions may not include `.git`, so git SHA discovery must not be mandatory. | Epic owner | Known |
| GitHub release jobs need to propagate the right commit SHA consistently across matrix builds. | Epic owner | Known |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `sift --version` prints a dev-suffixed version with a seven-character SHA on ordinary local builds.
- [ ] Release/dist builds print the plain package semver with a seven-character SHA and no `-dev` suffix.
- [ ] Builds without git metadata still succeed and report `unknown` as the SHA.
- [ ] The GitHub release workflow and release docs explicitly describe the build metadata contract.
<!-- END SUCCESS_CRITERIA -->
