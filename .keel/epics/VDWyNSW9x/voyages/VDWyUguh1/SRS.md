# Prepare 0.2.0 Release - Software Requirements Specification

> Bump the repository to 0.2.0 and verify the existing release path is ready for the v0.2.0 tag.

**Epic:** [VDWyNSW9x](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Updating the crate version to `0.2.0`.
- [SCOPE-02] Updating release-facing documentation examples that still embed the old version.
- [SCOPE-03] Verifying the release cut locally through the repository quality gate and cargo-dist planning for the `v0.2.0` tag.

### Out of Scope

- [SCOPE-04] Pushing commits or tags to the remote.
- [SCOPE-05] Changing release workflow behavior unless verification exposes a targeted defect.
- [SCOPE-06] Shipping unrelated feature or packaging changes as part of the release cut.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| `Cargo.toml` remains the canonical source of the crate semver. | dependency | The release cut could stamp conflicting versions across tools and binaries. |
| The existing cargo-dist configuration can plan a `v0.2.0` release without workflow redesign. | dependency | The release cannot be considered ready until the configuration issue is fixed. |
| `just check` remains the repository quality gate for release-readiness checks. | dependency | Another verification path would need to be substituted before acceptance. |

## Constraints

- The release cut must stay atomic and limited to versioning, release docs, and readiness verification.
- The verification evidence must include both the normal repository gate and a tag-oriented cargo-dist planning proof.
- Local version output must reflect the bumped semver after the cut.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The repository must declare `0.2.0` as the `sift` crate version in `Cargo.toml`, and the local CLI version output must reflect the bumped semver. | SCOPE-01,SCOPE-03 | FR-01 | command proof + inspection |
| SRS-02 | Release-facing documentation must replace stale embedded 0.1.x release examples with 0.2.0 values. | SCOPE-02 | FR-02 | command proof + inspection |
| SRS-03 | The repository quality gate must pass after the 0.2.0 version cut. | SCOPE-03 | FR-03 | command proof |
| SRS-04 | `cargo dist plan --tag v0.2.0` must succeed against the bumped repository state. | SCOPE-03 | FR-03 | command proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-05 | The release cut must remain auditable as a narrow version-preparation slice without unrelated source changes. | SCOPE-01,SCOPE-02,SCOPE-03 | NFR-01 | inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
