# Embeddable Library and Executable Packaging - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Determine the recommended packaging strategy for shipping `sift` as an embeddable Rust library without regressing the existing executable UX. | board: VDVQurZER |
| MG-02 | Define the canonical public API boundary required for embedding so downstream crates do not depend on accidental internals. | board: VDVQurZER |
| MG-03 | Preserve the current single-package install and release ergonomics unless research proves a workspace split is necessary. | manual: Recommendation and follow-up plan reviewed against current `cargo install` and release contract |

## Constraints

- Maintain the repository's "single Rust binary" and "no external database/daemon" operating contract.
- Prefer one canonical library entrypoint over exposing every internal module as public API.
- Avoid packaging churn that would break the existing `sift` executable name, release pipeline, or Homebrew/install story without strong evidence.
- Treat heavy capabilities such as evaluation tooling, dense models, and LLM reranking as opt-in boundaries if they complicate embedding.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when all MG-* goals with `board:` verification are satisfied
- YIELD to human when only `metric:` or `manual:` goals remain
