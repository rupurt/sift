# Example Consumer CLI - Software Requirements Specification

> Provide a runnable sift-embed example crate, a just workflow to invoke it, and docs that treat it as the canonical library-consumer reference.

**Epic:** [VDVkH5a6M](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Add a standalone example crate that builds a `sift-embed` CLI consumer from another Cargo package.
- [SCOPE-02] Add root-level `just` recipes that build and run the example consumer.
- [SCOPE-03] Update repository docs to point at the example as the canonical embedding reference.

### Out of Scope

- [SCOPE-04] Converting the repository into a Cargo workspace for the example crate.
- [SCOPE-05] Matching the full `sift` executable feature set inside the example consumer.
- [SCOPE-06] Shipping `sift-embed` as an independently released binary.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The crate-root facade (`Sift`, `SearchInput`, `SearchOptions`, and related root exports) is sufficient to implement a minimal search CLI. | dependency | The voyage would need follow-on facade work before the example could stay on the supported path. |
| A standalone manifest under `examples/` can depend on the root crate with a path dependency and still fit the single-package rollout. | assumption | The voyage would need a packaging redesign instead of a lightweight example package. |
| Repo verification will continue to be limited by the known linker issue for fully linked binaries in this environment. | dependency | Runtime command proofs may need to stay compile- and source-based for now. |

## Constraints

- Keep the example on the supported crate-root API; do not document or depend on `sift::internal` from the example crate.
- Preserve the root `sift` package as the main library/binary package.
- Keep the example command surface intentionally small and readable.
- Use repo-native verification paths: Cargo compile checks, `just` recipe inspection, and documentation review.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The repository must include a standalone example crate under `examples/` that builds an executable named `sift-embed` and routes `search` through crate-root `sift` facade types. | SCOPE-01 | FR-01 | `cargo check --manifest-path examples/sift-embed/Cargo.toml` + source inspection |
| SRS-02 | The repo root must expose `just` recipes that build and run the example consumer without requiring contributors to remember the manifest path. | SCOPE-02 | FR-02 | `just` recipe inspection + command proof |
| SRS-03 | Repository documentation must identify the example consumer as the canonical runnable embedding reference and show how to invoke `sift-embed search "<term>"`. | SCOPE-03 | FR-03 | docs review + example inspection |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-04 | The example delivery must preserve the single-package rollout by keeping the root package metadata unchanged and avoiding a workspace split or internal-only dependency path. | SCOPE-01 | NFR-01 | `cargo check --all-targets` + manifest inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
