# Version Output Metadata - Software Requirements Specification

> Stamp the CLI version output with semver and git SHA for local dev and release builds.

**Epic:** [VDWjylxX2](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Build-time generation of the canonical `sift --version` string.
- [SCOPE-02] Development vs release/dist suffix handling for the binary version output.
- [SCOPE-03] GitHub release workflow wiring for authoritative commit SHA injection.
- [SCOPE-04] Release documentation for the version metadata contract.

### Out of Scope

- [SCOPE-05] Adding a dedicated runtime command beyond clap `--version`.
- [SCOPE-06] Changing crate/package versioning away from `Cargo.toml`.
- [SCOPE-07] Extending version stamping to auxiliary example binaries.

## Assumptions & Dependencies

<!-- What we assume to be true; external systems, services, or conditions we depend on -->

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Cargo exposes the active build profile to `build.rs`. | dependency | Release-vs-dev suffix selection would need a different signal. |
| GitHub Actions exposes the build commit through `github.sha`. | dependency | Release jobs would need another explicit SHA source. |

## Constraints

- Version metadata must be embedded at compile time rather than fetched at runtime.
- The user-facing `--version` output must stay in the single-line clap format `sift <version> (<sha>)`.
- Fallback behavior for missing git metadata must be deterministic and testable.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The binary must render `sift <semver>-dev (<sha>)` for non-release builds, sourcing `<semver>` from `Cargo.toml` and normalizing the SHA to seven characters. | SCOPE-01,SCOPE-02 | FR-01 | automated test + CLI proof |
| SRS-02 | Release and dist-profile builds must render `sift <semver> (<sha>)` without the `-dev` suffix while using the same SHA normalization rules. | SCOPE-01,SCOPE-02 | FR-02 | automated test + CLI proof |
| SRS-03 | The GitHub release workflow and release documentation must define and apply the commit-SHA injection contract used by release builds. | SCOPE-03,SCOPE-04 | FR-03 | command proof + inspection |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-04 | Builds without git metadata must still compile and render `unknown` as the SHA without invoking git at runtime. | SCOPE-01,SCOPE-02 | NFR-01 | automated test + inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
