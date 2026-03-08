# Stabilize Zvec Build Integration - Product Requirements

## Problem Statement

`sift` cannot build because `zvec-sys` 0.3.0 is incompatible with the default
native toolchain exposed by the repository shell. The failure starts at CMake 4,
continues at GCC 15 in vendored RocksDB headers, and then surfaces a wrapper API
mismatch against `zvec v0.2.0`.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Restore a working local build path for `zvec-sys` in the repository development shell. | `nix develop --command cargo check` succeeds on Linux without manual tool overrides. | 100% success for the default developer workflow |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | A contributor building `sift` from the repository root. | A development shell and dependency setup that can compile vendored `zvec` sources without manual toolchain workarounds. |

## Scope

### In Scope

- [SCOPE-01] Pin the development shell to the minimum compatible native toolchain components required by `zvec-sys`.
- [SCOPE-02] Vendor the minimum `zvec-sys` compatibility patch required to build against `zvec v0.2.0`.
- [SCOPE-03] Verify the resulting repository state by running the default `cargo check` workflow.

### Out of Scope

- [SCOPE-04] Replacing `zvec-sys` entirely or implementing search features on top of the bindings.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The repository development shell must provide the native build components required for `zvec-sys` to compile successfully. | GOAL-01 | must | The current shell exposes incompatible CMake and GCC versions for the vendored native build. |
| FR-02 | The repository must carry the minimum `zvec-sys` patch needed to match the `zvec v0.2.0` API and avoid shared native build state. | GOAL-01 | must | A shell-only fix still fails inside the upstream wrapper and shared cargo-registry build paths. |
| FR-03 | The default developer verification command must complete without requiring a manual PATH or environment override. | GOAL-01 | must | The fix should live in repository configuration, not in ad hoc operator knowledge. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Limit compatibility work to the specific native build components and crate patch surface required for a working build. | GOAL-01 | must | Avoids broad package drift while restoring build compatibility. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Problem outcome | CLI proof in the repository development shell | Successful `nix develop --command cargo check` output recorded on the implementation story |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| `zvec-sys` 0.3.0 remains the intended binding strategy for the current codebase. | The compatibility patch may be unnecessary if the dependency is being replaced imminently. | Confirm by keeping the scope limited to the current dependency set in `Cargo.toml`. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| The vendored `zvec` build is large and may expose further upstream issues beyond the current compatibility patch. | Implementer | Mitigated by full `cargo check` verification |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] A developer can enter `nix develop` and run `cargo check` successfully without manual toolchain overrides.
<!-- END SUCCESS_CRITERIA -->
