# Setup Cargo Dist And Release Workflow - SDD

> Implement automated release pipeline using cargo-dist and GitHub Actions.

## Architecture Overview

The release pipeline leverages `cargo-dist`, a tool specifically designed for orchestrating Rust project releases. It integrates directly with `Cargo.toml` and generates a GitHub Actions workflow that handles the heavy lifting of building on multiple OSs and uploading artifacts.

## Components

### `Cargo.toml` (Metadata)
Holds the `cargo-dist` configuration, including target architectures, installer types, and CI provider details.

### `.github/workflows/release.yml`
An auto-generated (and potentially customized) workflow that triggers on tag pushes. It uses `dist` to build, package, and release.

## Data Flow

1. **Tag Push:** Developer pushes a tag (e.g., `git tag v0.1.0 && git push --tags`).
2. **Workflow Trigger:** GitHub Actions detects the tag and starts the `release.yml` workflow.
3. **Orchestration:** The `plan` job determines what needs to be built.
4. **Parallel Build:** Separate jobs for Linux, macOS, and Windows build the binaries using the `--release` profile.
5. **Packaging:** Artifacts are bundled into tarballs/zips and installers (.deb, .dmg, etc.) are created.
6. **Release Creation:** Artifacts and checksums are uploaded to a new GitHub Release.

## Design Approach

### cargo-dist Configuration

We will add the following to `Cargo.toml`:
- `targets`: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`.
- `installers`: `deb`, `rpm`, `dmg`, `msi`.
- `ci`: `github`.

## Deployment Strategy

- Update `Cargo.toml`.
- Generate/Create `release.yml`.
- Verify with a test tag.
