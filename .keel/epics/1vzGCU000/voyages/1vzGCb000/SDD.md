# Restore Zvec Build Compatibility - Software Design Description

> Make the development shell and vendored `zvec-sys` build path compatible with `zvec v0.2.0` so `cargo check` can complete successfully.

**SRS:** [SRS.md](SRS.md)

## Overview

The repository keeps its primary toolchain on `nixos-unstable`, but imports a
second pinned `nixpkgs` snapshot for the specific native build components that
`zvec-sys` needs: a pre-4.0 `cmake`, a GCC 14 wrapper, and the matching runtime
libraries. A repo-local `zvec-sys` patch then builds `zvec` in isolated
`OUT_DIR` paths, forwards the active compilers to CMake, derives bindgen include
paths from the shell toolchain, and updates the wrapper call site that no longer
matches `zvec v0.2.0`.

## Context & Boundaries

```
┌───────────────────────────────────────────────┐
│                  This Voyage                  │
│                                               │
│   flake.nix -> compat cmake + gcc14 shell    │
│                     |                         │
│                     v                         │
│   third_party/zvec-sys build.rs + wrapper    │
│                     |                         │
│                     v                         │
│                cargo check / sift            │
└───────────────────────────────────────────────┘
                     |
                     v
              vendored zvec v0.2.0
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `nixpkgs` | flake input | Primary repo toolchain and libraries | `nixos-unstable` |
| `nixpkgs-cmake` | flake input | Compatibility source for pre-4.0 `cmake` and GCC 14 toolchain components | `nixos-25.05` snapshot |
| `zvec-sys` | Rust crate | Builds vendored `zvec` sources as part of `cargo check` | `0.3.0` |
| `zvec` | upstream C++ library | Native vector engine compiled by `zvec-sys` | `v0.2.0` |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Compatibility strategy | Pin only the required native build components from a secondary Nixpkgs input | Solves the CMake 4 and GCC 15 incompatibilities without downgrading the full shell. |
| Crate strategy | Vendor the minimum `zvec-sys` patch surface in-repo | Fixes the wrapper API mismatch and eliminates shared cargo-registry/native-build state. |
| Verification path | Use the default `cargo check` workflow | Proves the fix works for the normal developer entrypoint. |

## Architecture

- `flake.nix` imports both the primary unstable package set and a secondary compatibility package set.
- The dev shell injects `cmake` and GCC 14 from the compatibility set, exports `CC` and `CXX`, and keeps the rest of the packages on the primary set.
- `Cargo.toml` patches `zvec-sys` to a repo-local copy under `third_party/zvec-sys`.
- The vendored `build.rs` clones `zvec v0.2.0` into `OUT_DIR`, configures isolated build trees, forwards the active compilers to CMake, and feeds bindgen the shell's effective include paths.
- The vendored wrapper source matches the `zvec v0.2.0` `Collection::AddColumn` signature.

## Components

- `flake.nix`: declares the secondary input, exposes compatibility `cmake` and GCC 14, and exports `CC` and `CXX`.
- `Cargo.toml`: patches `zvec-sys` to the repo-local vendored copy.
- `third_party/zvec-sys/build.rs`: isolates the native build and bindgen configuration.
- `third_party/zvec-sys/zvec-c-wrapper/src/collection.cpp`: matches the upstream `AddColumn` API.
- Board artifacts: capture the reason for the compatibility layer and the verification contract.

## Interfaces

- `nix develop --command cmake --version`
- `nix develop --command sh -lc 'echo $CC; $CC --version | head -n 1'`
- `nix develop --command cargo check`

## Data Flow

1. A developer enters the repository shell with `nix develop`.
2. The shell places the compatibility `cmake` binary on `PATH` and exports GCC 14 via `CC` and `CXX`.
3. `cargo check` invokes `zvec-sys`'s build script.
4. The build script clones `zvec v0.2.0` into `OUT_DIR`, configures isolated build trees with the compatibility compilers, and builds both `zvec` and the C wrapper.
5. Bindgen generates Rust FFI bindings using include paths derived from the active shell compiler.
6. Rust compiles the crate graph successfully without manual operator overrides.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| The compatibility toolchain input drifts or fails to resolve. | `nix develop`, `cmake --version`, or `$CC --version` fails. | Fix the flake input pin. | Refresh the lockfile and rerun shell verification. |
| Upstream `zvec-sys` changes its build layout or wrapper API again. | `cargo check` fails in the vendored build script or wrapper build. | Update the repo-local patch or drop it once upstream compatibility exists. | Verify with the default shell and record the new evidence. |
