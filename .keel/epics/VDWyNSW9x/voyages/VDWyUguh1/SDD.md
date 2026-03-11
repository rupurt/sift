# Prepare 0.2.0 Release - Software Design Description

> Bump the repository to 0.2.0 and verify the existing release path is ready for the v0.2.0 tag.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage treats the release cut as a narrow repository maintenance slice.
The implementation updates the canonical crate version, refreshes any
release-facing documentation that still embeds the old semver, and then proves
that the repository is ready for the `v0.2.0` tag by running the normal project
quality gate plus `cargo dist plan`.

## Context & Boundaries

### In Scope
- Update `Cargo.toml` and any coupled version metadata needed for the crate release.
- Refresh the main release guide so its examples and instructions match `0.2.0`.
- Run local release-readiness verification after the version cut.

### Out of Scope
- Publishing GitHub releases or pushing the release tag.
- Introducing workflow or feature changes unrelated to the release cut.
- Reworking cargo-dist configuration unless verification reveals a blocking defect.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │Cargo.toml│ │RELEASE.md│ │verify    │ │
│  │version   │ │examples  │ │commands  │ │
│  └──────────┘ └──────────┘ └──────────┘ │
└─────────────────────────────────────────┘
        ↑                    ↑
   [cargo/clap]        [cargo-dist tag plan]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `Cargo.toml` package metadata | repository contract | Source of truth for the release semver used by the crate and CLI version output | current crate metadata |
| `just check` | repository workflow | Proves formatting, linting, tests, and docs remain green after the cut | repo just recipe |
| `cargo dist plan` | release tooling | Proves the existing tag-based release workflow can plan the `v0.2.0` release | cargo-dist 0.22.1 |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Release cut scope | One narrow story updates version metadata and records release-readiness proofs. | Keeps the cut auditable and easy to revert if needed. |
| Release verification | Use `just check` and `cargo dist plan --tag v0.2.0` as the acceptance proof set. | This exercises both day-to-day repo health and the actual tag-based release path. |
| Remote publication | Leave push/tag publication out of scope for this slice. | The repository can be prepared and verified locally without assuming remote access. |

## Architecture

No runtime architecture changes are required. The work stays at the repository
configuration layer: version metadata in `Cargo.toml`, release documentation in
`RELEASE.md`, and release-readiness validation through existing local commands.

## Components

- `Cargo.toml`: bumped to `0.2.0` as the canonical release semver.
- `RELEASE.md`: refreshed so release instructions and examples match `0.2.0`.
- Verification commands: confirm the updated repository remains healthy and the release pipeline can plan `v0.2.0`.

## Interfaces

- User-facing CLI proof:
  - `cargo run -- --version`
- Repository verification:
  - `just check`
- Release planning:
  - `cargo dist plan --tag v0.2.0`

## Data Flow

1. Update the crate semver in `Cargo.toml`.
2. Refresh any release-facing documentation examples that still reference the old semver.
3. Run `cargo run -- --version` to confirm the local CLI reflects `0.2.0`.
4. Run `just check` to verify the repository after the cut.
5. Run `cargo dist plan --tag v0.2.0` to verify the tag-based release workflow.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| A stale 0.1.x version remains in release-facing docs | Grep or inspection proof fails | Update the stale reference before acceptance | Re-run the documentation proof |
| The version bump breaks repository verification | `just check` fails | Treat the release cut as incomplete | Fix the blocking issue before release acceptance |
| The tag-based release plan is misconfigured | `cargo dist plan --tag v0.2.0` fails | Stop the release cut and fix the release path | Re-run cargo-dist planning after the fix |
