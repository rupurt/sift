# Restore Zvec Build Compatibility - Software Requirements Specification

> Make the development shell and `zvec-sys` build path compatible with `zvec v0.2.0` so `cargo check` can complete successfully.

**Epic:** [1vzGCU000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

This voyage restores the default repository build path by pinning the native
toolchain components that `zvec-sys` actually requires and by vendoring the
crate patch needed to build against `zvec v0.2.0`.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| `zvec-sys` 0.3.0 shells out to `cmake` and builds against the active `CC`/`CXX` toolchain. | dependency | A shell-level compatibility pin would not affect the build if the crate stopped using the host toolchain. |
| `zvec-sys` 0.3.0's wrapper sources are not fully compatible with `zvec v0.2.0`. | dependency | A toolchain-only fix would still fail during wrapper compilation. |
| Contributors enter the repo through `nix develop` or `direnv`. | assumption | The fix would not cover workflows that bypass repository tooling entirely. |

## Constraints

- Keep the compatibility change repository-scoped and automatic.
- Preserve the existing unstable `nixpkgs` toolchain for the rest of the shell.
- Limit vendoring to the minimum `zvec-sys` patch surface required for a working build.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The development shell must expose a CMake 3.x binary before the main unstable toolchain's CMake 4 binary. | SCOPE-01 | FR-01 | `nix develop --command cmake --version` |
| SRS-02 | The development shell must export a GCC 14-compatible `CC` and `CXX` so the vendored `zvec` sources do not build against the incompatible GCC 15 wrapper. | SCOPE-01 | FR-01 | `nix develop --command sh -lc 'echo $CC; $CC --version | head -n 1'` |
| SRS-03 | The repository must vendor a `zvec-sys` patch that builds under isolated `OUT_DIR` paths and matches the `zvec v0.2.0` wrapper API. | SCOPE-02 | FR-02 | file inspection + `nix develop --command cargo check` |
| SRS-04 | `cargo check` must complete successfully from the default repository shell without manual `PATH`, `CC`, or `CXX` overrides. | SCOPE-02 | FR-02 | `nix develop --command cargo check` |
| SRS-05 | Compatibility pinning must be limited to the native build components required by `zvec-sys`, and the shell should avoid carrying unused alternate toolchains. | SCOPE-01 | NFR-01 | flake inspection + `cargo check` |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Compatibility pinning must be limited to the native build components required by `zvec-sys`, and the shell should avoid carrying unused alternate toolchains. | SCOPE-01 | NFR-01 | design reference only; story traceability is captured under `SRS-05` for Keel compatibility |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
