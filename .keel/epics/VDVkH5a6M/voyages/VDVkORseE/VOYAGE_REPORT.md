# VOYAGE REPORT: Example Consumer CLI

## Voyage Metadata
- **ID:** VDVkORseE
- **Epic:** VDVkH5a6M
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 2/2 stories complete

## Implementation Narrative
### Document The Runnable Embedded Example
- **ID:** VDVkxNEcJ
- **Status:** done

#### Summary
Update repository docs to present `sift-embed` as the canonical runnable
embedding example and show the repo-root commands that build or run it.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Repository documentation identifies the example crate as the canonical runnable embedding reference and shows `sift-embed search "<term>"` usage. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "sift-embed search|canonical runnable embedding reference|examples/sift-embed" README.md examples/sift-embed/README.md', SRS-03:start:end, proof: ac-1.log -->
- [x] [SRS-03/AC-02] Documentation shows the repo-root `just` recipes used to build or run the example consumer. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "just embed-build|just embed-search" README.md examples/sift-embed/README.md', SRS-03:start:end, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VDVkxNEcJ/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VDVkxNEcJ/EVIDENCE/ac-2.log)

### Add Sift-Embed Example Crate And Just Recipes
- **ID:** VDVkxNXc4
- **Status:** done

#### Summary
Add a standalone `examples/sift-embed` consumer package and repo-root `just`
recipes that demonstrate how another Rust crate builds and invokes `sift`
through the supported crate-root facade.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] The repository contains a standalone example crate under `examples/sift-embed` that builds an executable named `sift-embed` and routes `search` through crate-root `sift` facade types. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo metadata --format-version 1 --manifest-path examples/sift-embed/Cargo.toml --no-deps | rg -n "\"name\":\"sift-embed\"" && rg -n "^name = \"sift-embed\"$" examples/sift-embed/Cargo.toml && rg -n "^sift = \\{ path = \"../\\.\\.\" \\}$" examples/sift-embed/Cargo.toml && rg -n "override_usage = \"sift-embed search" examples/sift-embed/src/main.rs && rg -n "^use sift::\\{SearchInput" examples/sift-embed/src/main.rs && rg -n "Sift::builder\\(\\)\\.build\\(\\)" examples/sift-embed/src/main.rs', SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-02] The root `justfile` exposes recipes to build and run the example consumer from the repo root. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "^embed-build:|^embed-search" justfile', SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-04/AC-03] The example remains a standalone consumer package without a workspace split and does not depend on `sift::internal`. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! rg -n "^\\[workspace\\]" Cargo.toml && ! rg -n "sift::internal" examples/sift-embed/src/main.rs && rg -n "^\\[workspace\\]$" examples/sift-embed/Cargo.toml', SRS-04:start:end, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VDVkxNXc4/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VDVkxNXc4/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VDVkxNXc4/EVIDENCE/ac-3.log)


