# Library Facade and Packaging Cutover - Software Requirements Specification

> Define the first supported embedded API and packaging boundary while preserving the current executable contract

**Epic:** [VDVQurZER](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Introduce one canonical public facade for embedded search usage.
- [SCOPE-02] Remove CLI-specific parsing/rendering leakage from the supported library API.
- [SCOPE-03] Rewire the executable to consume the curated facade without changing its user-facing contract.
- [SCOPE-04] Make the stable public boundary explicit in exports and documentation.

### Out of Scope

- [SCOPE-05] Immediate workspace or multi-crate reorganization.
- [SCOPE-06] Retrieval-quality changes unrelated to the packaging cutover.
- [SCOPE-07] New model families, extraction formats, or evaluation methodology changes.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The current single-package layout is acceptable for the first supported library release. | assumption | The voyage would need re-planning around a package split. |
| Existing search/corpus services are reusable behind a narrower facade. | dependency | The implementation may need a deeper refactor than this voyage assumes. |
| CLI verification can continue through existing Rust test and command proofs. | dependency | Acceptance criteria would need different verification techniques. |

## Constraints

- Keep `sift` as the current executable name and preserve the existing install/release contract.
- Do not make internal adapters, presentation helpers, or CLI parsing types part of the supported embedded API by default.
- Prefer one canonical library path over compatibility shims or duplicate public entrypoints.
- Use repo-native verification paths: Rust tests, command proofs, and manual file inspection when appropriate.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The crate must expose one canonical high-level embedded search facade that allows another Rust project to execute a search without depending on internal modules directly. | SCOPE-01 | FR-01 | integration test + API inspection |
| SRS-02 | The supported public library API must not require `clap`-derived types, terminal rendering helpers, or other CLI-only concerns in its contract. | SCOPE-02 | FR-02 | cargo test + file inspection |
| SRS-03 | The `sift` executable must continue to build and expose the current command contract while consuming the curated library boundary. | SCOPE-03 | FR-03 | cargo test + `cargo run -- --help` + `cargo run -- search --help` |
| SRS-04 | The cutover must explicitly document which exports and modules are supported public API versus internal implementation detail. | SCOPE-04 | FR-04 | docs review + public export inspection |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-05 | The public API cutover must minimize avoidable semver surface area and package/release churn by keeping the initial rollout inside the existing package unless stronger evidence emerges. | SCOPE-01 | NFR-01 | module/export inspection + release-path review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
