---
id: VDVkxNXc4
title: Add Sift-Embed Example Crate And Just Recipes
type: feat
status: backlog
created_at: 2026-03-10T16:44:47
updated_at: 2026-03-10T16:45:26
scope: VDVkH5a6M/VDVkORseE
index: 2
---

# Add Sift-Embed Example Crate And Just Recipes

## Summary

Add a standalone `examples/sift-embed` consumer package and repo-root `just`
recipes that demonstrate how another Rust crate builds and invokes `sift`
through the supported crate-root facade.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] The repository contains a standalone example crate under `examples/sift-embed` that builds an executable named `sift-embed` and routes `search` through crate-root `sift` facade types. <!-- verify: cargo check --manifest-path examples/sift-embed/Cargo.toml, SRS-01:start:end -->
- [ ] [SRS-02/AC-02] The root `justfile` exposes recipes to build and run the example consumer from the repo root. <!-- verify: rg -n '^embed-build:|^embed-search' justfile, SRS-02:start:end -->
- [ ] [SRS-04/AC-03] The example remains a standalone consumer package without a workspace split and does not depend on `sift::internal`. <!-- verify: sh -lc '! rg -n "^\\[workspace\\]" Cargo.toml && ! rg -n "sift::internal" examples/sift-embed/src/main.rs', SRS-04:start:end -->
