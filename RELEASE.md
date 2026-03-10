# Release Process

This project uses [cargo-dist](https://opensource.axodotdev.com/cargo-dist/) to automate cross-platform releases. Binaries and installers for Linux, macOS, and Windows are automatically built and uploaded to GitHub Releases when a version tag is pushed.

## How to Install

### Homebrew (macOS and Linux)

```bash
brew tap rupurt/homebrew-tap
brew install sift
```

---

## How to Perform a Release

Follow these steps to release a new version of `sift`:

### 1. Update Version
Bump the version number in `Cargo.toml`. We follow [Semantic Versioning](https://semver.org/).

```toml
# Cargo.toml
[package]
name = "sift"
version = "0.1.1" # Update this
```

### 2. Commit and Push
Commit the version bump to the `main` branch.

```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.1.1"
git push origin main
```

### 3. Create and Push a Tag
Create a git tag corresponding to the new version (must start with `v`).

```bash
git tag v0.1.1
git push origin v0.1.1
```

### 4. Automated Workflow
Pushing the tag triggers the [Release GitHub Action](.github/workflows/release.yml). This workflow will:
- Plan the release using `cargo dist plan`.
- Build binaries for all supported platforms in parallel.
- Generate supported installers (shell, PowerShell, Homebrew, and `.msi`).
- Create a GitHub Release and upload all artifacts and checksums.

### 5. Verify the Release
Once the GitHub Action completes:
1.  Go to the [Releases](https://github.com/rupurt/sift/releases) page.
2.  Verify that all artifacts (tarballs and installers) are attached.
3.  Ensure the `checksums.txt` file is present.

---

## Supported Platforms & Artifacts

| Platform | Target Triple | Artifacts |
|----------|---------------|-----------|
| **Linux (x86_64, glibc)** | `x86_64-unknown-linux-gnu` | `.tar.gz`, shell installer |
| **Linux (x86_64, static)** | `x86_64-unknown-linux-musl` | `.tar.gz`, shell installer |
| **Linux (ARM64)** | `aarch64-unknown-linux-gnu` | `.tar.gz`, shell installer |
| **macOS (Intel)** | `x86_64-apple-darwin` | `.tar.gz`, shell installer, Homebrew formula |
| **macOS (Apple Silicon)** | `aarch64-apple-darwin` | `.tar.gz`, shell installer, Homebrew formula |
| **Windows (x86_64)** | `x86_64-pc-windows-msvc` | `.zip`, `.msi`, PowerShell installer |

---

## Local Testing

You can simulate the release plan locally (if `cargo-dist` is installed):

```bash
# See what would be built
cargo dist plan

# Build artifacts locally (outputs to target/dist)
cargo dist build
```

### Building a Static Binary Locally

On Linux, you can build a truly static binary using Nix:

```bash
just build-static
```

The resulting binary will be linked at `target/release/sift-static/bin/sift`.
