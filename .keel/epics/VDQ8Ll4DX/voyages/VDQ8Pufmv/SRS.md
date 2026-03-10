# Setup Cargo Dist And Release Workflow - SRS

> Configure cargo-dist and generate the GitHub Action for automated releases.

## Scope

### In Scope

- [SCOPE-01] Add `[package.metadata.dist]` configuration to `Cargo.toml`.
- [SCOPE-02] Create `.github/workflows/release.yml` with multi-platform jobs.
- [SCOPE-03] Configure installers for .deb, .rpm, .dmg, .msi.
- [SCOPE-04] Ensure release is triggered by `v*` tags.

### Out of Scope

- [SCOPE-05] Code signing.
- [SCOPE-06] Publishing to external repositories (e.g., Homebrew).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Verification |
|----|-------------|--------|-------|--------------|
| SRS-01 | `Cargo.toml` must contain `cargo-dist` metadata specifying targets for Linux, macOS, and Windows. | FR-01 | SCOPE-01 | board: VDQ8VatjT |
| SRS-02 | The release workflow must generate a GitHub Release with attached binaries and installers. | FR-02 | SCOPE-02 | manual: Push tag and check release |
| SRS-03 | Installers for .deb and .rpm must be generated for Linux. | FR-02 | SCOPE-03 | manual: Inspect artifacts |
| SRS-04 | A DMG installer must be generated for macOS. | FR-02 | SCOPE-03 | manual: Inspect artifacts |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Verification |
|----|-------------|--------|-------|--------------|
| SRS-05 | The release process should reuse existing build logic where possible to ensure consistency. | NFR-01 | SCOPE-02 | manual: Inspect workflow yaml |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Push a test tag `v0.1.0-test.1` and verify the full pipeline execution.
- Download and inspect artifacts from the resulting GitHub Release.
