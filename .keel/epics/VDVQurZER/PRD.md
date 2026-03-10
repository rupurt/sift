# Embeddable Library Packaging Research - Product Requirements

> `sift` can become a credible embeddable Rust library without an immediate
workspace split because the repository already ships a library target and the
CLI already composes it directly. The main missing piece is a deliberate public
API boundary: today the crate exposes a broad set of internal modules, request
types, and CLI-adjacent concerns that are acceptable for internal use but weak
as a semver-stable embedding contract.

## Problem Statement

The current codebase is close to "library plus executable" mechanically, but
not yet productized that way.

The repository already contains:

- a top-level `src/lib.rs`
- a `src/main.rs` that imports `sift::...`
- modular search, extraction, caching, and evaluation code

What it does not yet contain is a canonical public library surface. The crate
currently re-exports broad internal modules, some public search types carry
CLI-oriented derives or operational knobs, and the packaging story has not been
evaluated against third-party embedders.

The question is therefore not "should we rewrite `sift` into a library," but
"which packaging path produces a stable, ergonomic embedded API with the least
avoidable churn to the current executable and release flow."

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Publish a canonical embedded search API for Rust consumers | Delivery signal | A planned voyage and stories cover the public API cutover |
| GOAL-02 | Preserve the existing `sift` executable UX and packaging story during the cutover | Regression signal | CLI contract remains verifiable during implementation |
| GOAL-03 | Avoid unnecessary cargo/package churn until the curated API proves it is needed | Scope discipline | Workspace split remains explicitly deferred unless evidence changes |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Rust Embedder | Wants to call `sift` from another Rust project without copying internal modules | Stable, documented library API |
| CLI User | Uses `sift` as an executable today | No disruption to commands, installs, or release artifacts |
| Maintainer | Evolves retrieval internals quickly | Clear line between stable facade and private modules |

## Scope

### In Scope

- [SCOPE-01] Define the first supported public library facade for search.
- [SCOPE-02] Remove or quarantine CLI-specific concerns from the public library boundary.
- [SCOPE-03] Preserve the existing executable contract while rewiring it to use the curated facade.
- [SCOPE-04] Document what is stable public API versus what remains internal implementation detail.

### Out of Scope

- [SCOPE-05] Immediate workspace or multi-crate decomposition unless the facade-first plan fails.
- [SCOPE-06] Retrieval-quality changes unrelated to packaging and public API boundaries.
- [SCOPE-07] New model families, new extraction formats, or benchmark methodology changes.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The crate must expose one canonical high-level library entrypoint for running `sift` search from another Rust project. | GOAL-01 | must | Embedders need a supported API instead of ad hoc access to internals. |
| FR-02 | The supported library entrypoint must not require CLI parsing/rendering concerns or other terminal-specific helpers in its public contract. | GOAL-01 | must | External consumers should depend on search concepts, not CLI glue. |
| FR-03 | The existing `sift` executable must continue to build and preserve its user-facing command contract while consuming the curated library boundary. | GOAL-02 | must | The packaging cutover should not regress current CLI users. |
| FR-04 | The library packaging plan must explicitly document which modules and types are stable public surface versus internal implementation detail. | GOAL-01, GOAL-03 | should | This prevents accidental semver commitments. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The public API cutover should minimize avoidable semver surface area and package/release churn. | GOAL-03 | must | Preserves maintainer agility while the embedded API matures. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove functional behavior through story-level verification evidence mapped to voyage requirements.
- Validate non-functional posture with operational checks and documented artifacts.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Most embedder value can be unlocked without an immediate package split | The epic may need a follow-on workspace migration voyage | Validate after facade-first implementation |
| Current CLI behavior is the correct compatibility anchor during packaging changes | Additional migration work may be needed for release/install workflows | Re-verify with CLI proofs during implementation |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Do heavy capabilities such as reranking and rich extraction need feature gates in the first cutover? | Maintainer | Open |
| Is the eventual crates.io story still one package, or should the CLI become a dedicated package later? | Maintainer | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [x] The epic is grounded in a bearing-backed recommendation that favors a curated single-package facade before any workspace split.
- [x] The initial voyage decomposes the cutover into implementable public-API and CLI-boundary slices.
- [x] The scope explicitly preserves the existing executable contract while narrowing the supported library surface.
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

### Findings

- The repository is already structurally capable of serving as both library and
  executable because the binary composes `sift` as a crate today. [SRC-01]
  [SRC-05] [SRC-07]
- The current public API is broader than it should be because internal search
  modules and runtime concerns are re-exported directly. [SRC-02] [SRC-03]
  [SRC-04]
- A workspace split is not the first problem to solve. The first problem is
  defining one canonical embedded API instead of letting embedders reach into
  arbitrary internal modules. [SRC-02] [SRC-03]

### Opportunity Cost

The opportunity cost is spending time on packaging and API design instead of
retrieval quality, richer format support, or performance work. That trade is
worth making now because a stable library boundary will constrain how future
features are exposed, and the current repo already has the architectural
ingredients for a clean cutover. [SRC-05] [SRC-06]

### Dependencies

- A deliberate top-level facade must replace broad module re-exports. [SRC-02]
  [SRC-03]
- CLI-only concerns should stop leaking into public library types and helpers.
  [SRC-04] [SRC-05]
- The release/install story should remain anchored on the current `sift`
  package unless later evidence proves separation is necessary. [SRC-01]

### Alternatives Considered

- Keep the current public surface and simply document it.
  Rejected because it would freeze too many internals as semver commitments.
  [SRC-02] [SRC-03]
- Split into a workspace immediately.
  Deferred because the current repo already works as one package and the API
  boundary is the sharper problem. [SRC-01] [SRC-05]
- Introduce several internal crates now.
  Rejected for the next phase because it adds the most churn before the desired
  external API is even validated. [SRC-03] [SRC-04]

---

*This PRD was seeded from bearing `VDVQurZER`. See `bearings/VDVQurZER/` for original research.*
