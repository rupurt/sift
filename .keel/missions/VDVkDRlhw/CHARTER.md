# Runnable Embedded Example CLI - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Ship a runnable example crate named `sift-embed` that exposes a `search` command and demonstrates consuming the supported `sift` library facade from another Rust package. | board: VDVkH5a6M |
| MG-02 | Add repository-native documentation and a `just` workflow so contributors can build or run the example consumer without inventing their own integration path. | board: VDVkH5a6M |

## Constraints

- Keep the primary `sift` package as the published library/binary package; do not turn this into a workspace split unless the example absolutely requires it.
- The example crate must depend on the supported crate-root facade rather than `sift::internal`.
- The example command surface should stay intentionally small: `sift-embed search "term"` should work against the current directory by default.
- Treat the example as documentation-grade reference code: easy to run, easy to read, and covered by repo-native verification.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when all MG-* goals with `board:` verification are satisfied
- YIELD to human when only `metric:` or `manual:` goals remain
