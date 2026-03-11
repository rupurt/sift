# Version Output Metadata - Software Design Description

> Stamp the CLI version output with semver and git SHA for local dev and release builds.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds a build-time metadata step that computes one canonical version
string for the CLI. `build.rs` resolves the package version, build channel, and
git SHA, then exports the final string through `cargo:rustc-env` for clap to
render. Release artifacts receive the authoritative SHA from GitHub Actions,
while local builds fall back to git discovery and ultimately `unknown`.

## Context & Boundaries

### In Scope
- Build-time metadata generation for `sift`
- Clap version output wiring
- GitHub Actions release-job environment wiring
- Release documentation updates

### Out of Scope
- Runtime git inspection or new version subcommands
- Stamping auxiliary example binaries
- Changing semver/tagging policy

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │build.rs │  │clap CLI │  │release  │ │
│  │metadata │  │version  │  │workflow │ │
│  └─────────┘  └─────────┘  └─────────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   [git / env]     [Cargo profiles]
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Cargo build scripts | toolchain | Export compile-time env vars into the crate build | stable build.rs contract |
| clap derive metadata | library | Render the final single-line `--version` output | clap 4 |
| GitHub Actions `github.sha` | CI metadata | Provide deterministic release commit SHA | GitHub Actions context |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Build channel detection | Treat `release` and `dist` profiles as release builds; other profiles remain dev. | This matches local cargo behavior and cargo-dist artifact builds without requiring runtime state. |
| SHA source order | Prefer explicit env, then local git lookup, then `unknown`. | Release jobs are deterministic, local builds remain convenient, and source-only trees still compile. |
| Output contract | Export one final version string to clap instead of building it at runtime. | Keeps `--version` simple and avoids runtime git dependencies. |

## Architecture

`build.rs` owns build metadata resolution. A small shared formatting module
defines how semver, dev suffixes, and SHA normalization compose into the final
string. `src/main.rs` consumes the generated `SIFT_CLI_VERSION` env var and
hands it to clap. The release workflow injects `SIFT_GIT_SHA` during cargo-dist
artifact builds so release binaries stamp the exact commit even in shallow or
source-packaged environments.

## Components

- `build.rs`: resolves build profile, commit SHA, and final version string; emits rerun hints for git/env changes.
- Shared version formatting module: centralizes `-dev` vs release formatting and `unknown` fallback behavior for tests.
- `src/main.rs`: passes the precomputed version string into clap.
- `.github/workflows/release.yml`: exports the authoritative release SHA into cargo-dist build jobs.
- `RELEASE.md`: documents the local vs release versioning contract.

## Interfaces

- Build env inputs:
  - `PROFILE`
  - optional `SIFT_GIT_SHA`
- Build env output:
  - `SIFT_CLI_VERSION`
- CLI surface:
  - `sift --version` -> `sift <version> (<sha>)`

## Data Flow

1. Cargo invokes `build.rs`.
2. `build.rs` reads the package version and active profile.
3. `build.rs` resolves a SHA from `SIFT_GIT_SHA`, then git, then `unknown`.
4. Shared formatting logic produces either `<semver>-dev (<sha>)` or `<semver> (<sha>)`.
5. `build.rs` exports `SIFT_CLI_VERSION`.
6. Clap renders `sift <SIFT_CLI_VERSION>` for `--version`.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Git metadata unavailable | git command fails or `.git` is missing | Use `unknown` SHA | Build still succeeds with deterministic fallback output |
| GitHub Actions SHA env missing | Release workflow inspection or release-proof command fails | Release artifacts would show fallback or local git SHA | Verification catches the workflow gap before acceptance |
| Build profile not explicitly release/dist | `PROFILE` inspection in build.rs | Treat as dev | Keeps local debug/test builds on the safe default path |
