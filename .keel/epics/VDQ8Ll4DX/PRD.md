# Automated Cross-Platform Release Pipeline - Product Requirements

> This epic focuses on establishing a robust, automated release pipeline for `sift` using `cargo-dist`. We will provide pre-built binaries and installers for Linux, macOS, and Windows, triggered by version tags in Git.

## Problem Statement

Currently, `sift` must be built from source by users. To reach a wider audience, we need to provide easily installable artifacts (tarballs, .deb, .rpm, .dmg, .msi) for all major operating systems. The release process should be automated to ensure consistency and reduce manual overhead.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Automate multi-platform builds | Successful builds on Linux, macOS, Windows | 100% build success rate in CI |
| GOAL-02 | Provide various installers | Availability of .deb, .rpm, .dmg, .msi | All specified formats available in GitHub Release |
| GOAL-03 | Connect release to versioning | Automatic release generation on tag push | Tag `vX.Y.Z` triggers a full release |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | Wants to use `sift` locally | Fast and easy installation on their OS. |
| DevOps Engineer | Wants to integrate `sift` into pipelines | Stable binary downloads and package manager support. |

## Scope

### In Scope

- [SCOPE-01] Integration of `cargo-dist` into `Cargo.toml`.
- [SCOPE-02] Generation and customization of GitHub Action for releases.
- [SCOPE-03] Configuration of artifacts: Linux (tar.gz, deb, rpm), macOS (tar.gz, dmg), Windows (zip, msi).
- [SCOPE-04] Integration with crate version number.

### Out of Scope

- [SCOPE-05] Publishing to package managers like Homebrew, Chocolatey, or Crates.io (deferred to future epics).
- [SCOPE-06] Code signing for macOS and Windows (initial release will be unsigned).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The build system must generate executables for x86_64 and aarch64 (where applicable). | GOAL-01 | must | Ensures broad hardware compatibility. |
| FR-02 | The release process must be triggered by pushing a git tag following the `v*` pattern. | GOAL-03 | must | Standard practice for automated releases. |
| FR-03 | GitHub Releases must include a checksum file for all artifacts. | GOAL-02 | must | Ensures artifact integrity for users. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Release builds must use the `--release` profile for maximum performance. | GOAL-01 | must | `sift` performance is critical for its value proposition. |
| NFR-02 | The release workflow must not take more than 30 minutes to complete. | GOAL-01 | should | Efficient CI usage. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Push a test tag (e.g., `v0.1.0-rc.1`) and verify the GitHub Action triggers and completes successfully.
- Manually inspect the generated artifacts in the draft release.
- Test installers on respective platforms (if environments are available).

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| `cargo-dist` can handle multi-architecture builds for Linux and macOS. | We might need manual cross-compilation setup. | Check `cargo-dist` documentation and trial runs. |
| GitHub Actions runners are sufficient for all target builds. | We might need to provide custom runners. | Monitor build times and success rates. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Will unsigned DMG and MSI installers cause issues for users? | Product | Open. Initial release will be unsigned. |
| Should we publish to crates.io as part of this epic? | Product | Deferred to future work. |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `cargo-dist` configured in `Cargo.toml`.
- [ ] `release.yml` generated in `.github/workflows`.
- [ ] Successful cross-platform build triggered by a tag.
- [ ] Checksums and artifacts available in GitHub Release.
<!-- END SUCCESS_CRITERIA -->
