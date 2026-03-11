# Stamped CLI Version Metadata - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Ship `sift --version` output that reports the package semver and a short git SHA, using a `-dev` suffix for non-release builds and `unknown` when git metadata is unavailable. | board: VDWjylxX2 |
| MG-02 | Ensure release artifacts built through the current release pipeline drop the `-dev` suffix while still stamping the exact build commit. | board: VDWjylxX2 |

## Constraints

- Keep the existing cargo-dist and GitHub Actions release flow as the authoritative release path.
- Resolve version metadata at build time; do not add runtime git process calls to `sift --version`.
- Preserve the user-facing format exactly as `sift <version> (<sha>)`.
- Treat short SHAs as seven hexadecimal characters and fall back to `unknown` when git metadata is unavailable.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when all MG-* goals with `board:` verification are satisfied
- YIELD to human when only `metric:` or `manual:` goals remain
