# Public Release Preparation - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Configure `cargo-dist` in `Cargo.toml` for multi-platform releases | board: VDQ8Ll4DX |
| MG-02 | Generate GitHub Action for the release process | board: VDQ8Ll4DX |
| MG-03 | Support for Linux (tar.gz, deb, rpm), macOS (tar.gz, dmg), and Windows (zip, msi) | board: VDQ8Ll4DX |
| MG-04 | Release workflow is triggered by version-tagging (e.g., v0.1.0) | manual: Workflow triggered on tag push |

## Constraints

- Ensure the release process is fully automated via GitHub Actions.
- Maintain consistency with the current project structure and CI.
- Use standard `cargo-dist` patterns for installers and artifacts.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when all MG-* goals with `board:` verification are satisfied
- YIELD to human when only `metric:` or `manual:` goals remain
