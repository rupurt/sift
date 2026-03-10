---
id: VDVkxNXc4
title: Add Sift-Embed Example Crate And Just Recipes
type: feat
status: in-progress
created_at: 2026-03-10T16:44:47
updated_at: 2026-03-10T16:46:04
scope: VDVkH5a6M/VDVkORseE
index: 2
started_at: 2026-03-10T16:46:05
---

# Add Sift-Embed Example Crate And Just Recipes

## Summary

Add a standalone `examples/sift-embed` consumer package and repo-root `just`
recipes that demonstrate how another Rust crate builds and invokes `sift`
through the supported crate-root facade.

## Acceptance Criteria

- [x] [SRS-01/AC-01] The repository contains a standalone example crate under `examples/sift-embed` that builds an executable named `sift-embed` and routes `search` through crate-root `sift` facade types. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo metadata --format-version 1 --manifest-path examples/sift-embed/Cargo.toml --no-deps | rg -n "\"name\":\"sift-embed\"" && rg -n "^name = \"sift-embed\"$" examples/sift-embed/Cargo.toml && rg -n "^sift = \\{ path = \"../\\.\\.\" \\}$" examples/sift-embed/Cargo.toml && rg -n "override_usage = \"sift-embed search" examples/sift-embed/src/main.rs && rg -n "^use sift::\\{SearchInput" examples/sift-embed/src/main.rs && rg -n "Sift::builder\\(\\)\\.build\\(\\)" examples/sift-embed/src/main.rs', SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-02] The root `justfile` exposes recipes to build and run the example consumer from the repo root. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "^embed-build:|^embed-search" justfile', SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-04/AC-03] The example remains a standalone consumer package without a workspace split and does not depend on `sift::internal`. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! rg -n "^\\[workspace\\]" Cargo.toml && ! rg -n "sift::internal" examples/sift-embed/src/main.rs && rg -n "^\\[workspace\\]$" examples/sift-embed/Cargo.toml', SRS-04:start:end, proof: ac-3.log -->
