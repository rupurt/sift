---
created_at: 2026-03-10T15:51:43
---

# Reflection - Decouple Public API From CLI Concerns

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VDVXb0grh: Title
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

The public embedded API can stay stable without exposing the internal
`*Policy` enums that currently carry `clap` derives. A thin facade-layer enum
set is enough to preserve current behavior while keeping the supported library
contract free of executable-specific parsing concerns.

Driving the change from the integration test first was useful because the
failure mode was concrete: `sift::Retriever` did not exist at the root until
the facade was introduced. That made the public contract gap explicit before
touching the implementation.

`nix develop -c just check` still does not complete in this environment. The
failure is outside this story slice: `cargo nextest run` reaches the linker and
then fails under `mold` with unresolved symbols including `open64`,
`fstat64`, `lseek64`, and `gnu_get_libc_version`.
