---
created_at: 2026-03-10T16:03:14
---

# Reflection - Document The Supported Embedded API Boundary

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VDVaUf5cQ: Title
| Field | Value |
|-------|-------|
| **Category** | code/testing/process/architecture |
| **Context** | describe when this applies |
| **Insight** | the fundamental discovery |
| **Suggested Action** | what to do next time |
| **Applies To** | file patterns or components |
| **Linked Knowledge IDs** | optional canonical IDs this insight builds on |
| **Observed At** | RFC3339 timestamp (e.g. 2026-02-22T12:00:00Z) |
| **Score** | 0.0-1.0 (impact significance) |
| **Confidence** | 0.0-1.0 (insight quality) |
| **Applied** | |
-->

## Observations

The documentation slice was stronger once the boundary became real in code.
Moving the old wide surface behind `sift::internal` let the README and crate
docs describe a concrete supported contract instead of a convention that callers
could bypass accidentally.

An explicit internal namespace is a practical compromise for the single-package
rollout. The executable, benches, and repository-internal tests can keep using
low-level modules without forcing embedders to treat those modules as stable API.

`cargo check --all-targets` passed after the namespace cutover. Repo-level
verification is still blocked in this environment: `nix develop -c just check`
reaches `cargo nextest run` and then fails during linking under `mold` with
unresolved symbols including `open64`, `lseek64`, `gnu_get_libc_version`, and
related glibc-era entry points.
