# Release 0.2.0 - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Prepare the repository for the `0.2.0` release by updating the crate version and any user-facing release documentation that embeds the current version. | board: VDWyNSW9x |
| MG-02 | Verify that the repository is release-ready for the `v0.2.0` tag using the existing cargo-dist workflow and project quality gates. | board: VDWyNSW9x |

## Constraints

- Keep the current cargo-dist and GitHub Actions release flow as the only release path.
- Do not change the release workflow semantics as part of the version cut unless a verification failure forces a targeted fix.
- Treat remote publication as out of scope for this slice; prepare and verify the repository locally.
- Keep the release cut atomic so the version bump and release-readiness proof land together.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when all MG-* goals with `board:` verification are satisfied
- YIELD to human when only `metric:` or `manual:` goals remain
