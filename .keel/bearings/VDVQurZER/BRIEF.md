# Embeddable Library Packaging Research — Brief

## Hypothesis

`sift` can become a credible embeddable Rust library without an immediate
workspace split because the repository already ships a library target and the
CLI already composes it directly. The main missing piece is a deliberate public
API boundary: today the crate exposes a broad set of internal modules, request
types, and CLI-adjacent concerns that are acceptable for internal use but weak
as a semver-stable embedding contract.

## Problem Space

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

## Success Criteria

This research is valuable if it replaces a vague "make it embeddable" goal with
a concrete packaging recommendation and a phased cutover plan.

- [x] Compare the major packaging options: curated single-package facade,
  workspace split, and deeper internal multi-crate decomposition.
- [x] Identify the concrete coupling points in the current repo that block a
  stable embedded API.
- [x] Recommend a preferred rollout path, including what should change first
  and what should stay deferred until later.

## Open Questions

- Should the public library remain the `sift` crate, or should the executable
  become a dedicated `sift-cli` package later?
- Which heavy capabilities should become cargo features before the library is
  presented as a supported embedding surface?
- Is a single-package facade sufficient for the next phase, or does the release
  pipeline justify a workspace split immediately?
