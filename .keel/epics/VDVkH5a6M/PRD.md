# Runnable Embedded Example Consumer - Product Requirements

## Problem Statement

Embedders need a working reference CLI that consumes the supported sift library facade without reaching through accidental internals.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Give embedders a runnable reference consumer that demonstrates how another Rust crate should invoke `sift` as a library. | The repository contains a buildable `sift-embed` example crate that uses the crate-root facade only. | One canonical example crate checked into the repo |
| GOAL-02 | Make the example easy for contributors to discover and run from the repo root. | The repository exposes a `just` workflow and documentation that point to the example as the reference integration path. | One build recipe, one run recipe, and updated docs |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Library Evaluator | A developer deciding whether `sift` is usable as an embeddable Rust dependency. | A concrete reference consumer that proves the supported API is sufficient. |
| Repository Contributor | A maintainer updating the library facade or docs. | A fast repo-native way to build or run the example so regressions are obvious. |

## Scope

### In Scope

- [SCOPE-01] Add a standalone example crate that builds an executable named `sift-embed`.
- [SCOPE-02] Add repo-root `just` recipes to build and run the example consumer.
- [SCOPE-03] Update repository documentation to point at the example as the canonical embedded consumer reference.

### Out of Scope

- [SCOPE-04] Turning the repository into a Cargo workspace for this example alone.
- [SCOPE-05] Reproducing full `sift` CLI feature parity inside the example consumer.
- [SCOPE-06] Publishing or distributing `sift-embed` as a separately released artifact.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The repository must contain a runnable example crate that exposes `sift-embed search [PATH] <QUERY>` while consuming the supported crate-root facade. | GOAL-01 | must | Embedders need a real consumer package, not just snippets. |
| FR-02 | The repo root must provide a `just` workflow to build and run the example consumer without manually assembling Cargo commands. | GOAL-02 | must | Contributors should be able to verify the integration path quickly. |
| FR-03 | Repository documentation must identify the example consumer as the canonical embedding reference and show how to invoke it. | GOAL-01, GOAL-02 | must | The example only helps if readers can find and use it. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Preserve the current single-package rollout by keeping the root `sift` package intact and treating the example as a standalone consumer package rather than a workspace split. | GOAL-01, GOAL-02 | must | The example should reinforce the packaging recommendation, not undermine it. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Example crate | Cargo compile checks plus source inspection of facade usage | Story-level proof logs for the example manifest and CLI wiring |
| Repo workflow | `just` recipe inspection plus targeted command proofs | Story-level proof logs for build/run recipes |
| Documentation | Manual review plus grep-based contract checks | Story-level proof logs tied to README/example docs |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A standalone example manifest under `examples/` is sufficient and does not require a workspace split. | The epic would need packaging redesign instead of a simple consumer example. | Validate through Cargo manifest wiring during implementation. |
| The supported crate-root facade is mature enough to power a minimal search CLI without reaching through `sift::internal`. | The example would expose gaps in the embedded API and require follow-on facade work. | Validate through the example crate implementation. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| The current environment cannot fully link runnable binaries because of the existing `mold`/glibc issue. | Epic owner | Known |
| Example scope could creep toward full CLI parity if command shape is not kept intentionally small. | Epic owner | Mitigated in scope |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] A contributor can point at one checked-in `sift-embed` package as the reference embedding consumer.
- [ ] The repo root exposes a documented `just` path to build or run the example consumer.
- [ ] The example and docs reinforce the crate-root facade as the supported API surface without introducing a workspace split.
<!-- END SUCCESS_CRITERIA -->
