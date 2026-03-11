# Release 0.2.0 Cut - Product Requirements

## Problem Statement

The repository is still versioned as 0.1.0, so the next public tag would ship
stale semver metadata unless the 0.2.0 release cut is prepared and verified in
the repository first.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Prepare the repository metadata for the 0.2.0 release. | The crate and release-facing docs consistently refer to `0.2.0`. | One atomic repository change set for the 0.2.0 cut |
| GOAL-02 | Prove that the existing release path is ready for the `v0.2.0` tag. | The normal quality gates and a `cargo dist plan` run succeed for the 0.2.0 tag. | Local verification completes without release-pipeline regressions |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Release Maintainer | The contributor cutting the next GitHub release. | A clean, verified repository state that can be tagged as `v0.2.0` without manual cleanup. |
| Binary User | Anyone downloading the next released `sift` artifact. | Correct 0.2.0 version metadata in the shipped binary and release materials. |

## Scope

### In Scope

- [SCOPE-01] Bump the crate version from `0.1.0` to `0.2.0`.
- [SCOPE-02] Update release-facing documentation examples that still embed the old version.
- [SCOPE-03] Verify the existing release workflow locally using the repository quality gates and a `cargo dist plan` run for `v0.2.0`.

### Out of Scope

- [SCOPE-04] Publishing the GitHub release or pushing the release tag to the remote.
- [SCOPE-05] Feature work unrelated to the release cut.
- [SCOPE-06] Redesigning the cargo-dist or GitHub Actions release pipeline beyond targeted fixes required by verification.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The repository must declare `0.2.0` as the package version for the `sift` crate. | GOAL-01 | must | The release tag must correspond to the semver embedded in the binary and crate metadata. |
| FR-02 | Release-facing documentation must describe the 0.2.0 cut without stale 0.1.x examples. | GOAL-01 | must | Maintainers need accurate release instructions for the version being cut. |
| FR-03 | The existing release path must be validated locally for the `v0.2.0` tag through project verification and cargo-dist planning. | GOAL-02 | must | A version bump alone is not enough; the release should be proven ready before tagging. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The 0.2.0 release cut must remain atomic and must not introduce unrelated workflow or product changes. | GOAL-01, GOAL-02 | must | Release preparation should be easy to audit and safe to ship. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Version metadata | File inspection plus `cargo run -- --version` | Story proof logs tied to `Cargo.toml` and CLI output |
| Release readiness | `just check` and `cargo dist plan --tag v0.2.0` | Story proof logs from repo verification and dist planning |
| Documentation accuracy | Targeted grep/file proof | Story proof logs tied to `RELEASE.md` |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing release workflow is already functionally correct and only needs the version cut. | Additional repair work would be required before a tag can be pushed. | Validate with local release planning during execution. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| `cargo dist plan` may reveal a latent release configuration issue unrelated to the version bump. | Epic owner | Known |
| Some docs may still embed stale semver examples outside the primary release guide. | Epic owner | Known |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The crate version and release-facing docs consistently refer to 0.2.0.
- [ ] `cargo run -- --version` reports the new 0.2.0 development version locally.
- [ ] `just check` passes after the version cut.
- [ ] `cargo dist plan --tag v0.2.0` succeeds locally.
<!-- END SUCCESS_CRITERIA -->
