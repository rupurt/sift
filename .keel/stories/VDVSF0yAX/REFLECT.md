---
created_at: 2026-03-10T15:46:50
---

# Reflection - Introduce Canonical Embedded Search Facade

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VDVWMr04D: Title
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

The existing search stack already had a usable orchestration seam in
`run_search`, so the supported facade could stay thin: `Sift::builder()`,
`SearchInput`, and `SearchOptions` now wrap the current runtime instead of
forking it. That keeps this first story focused on one canonical entrypoint
while deferring export cleanup to the next blocked stories.

The main blocker during verification is environmental rather than story-local.
`cargo fmt --check` and `cargo check --tests` passed, and the new
`library_facade_test` compiles via `cargo check --test library_facade_test`,
but `nix develop -c just check` still fails during `cargo nextest run` at the
link step with unresolved symbols such as `open64`, `fstat64`, `lseek64`, and
`gnu_get_libc_version` under `mold`. That linker issue needs separate cleanup
before full repo test execution can be used as story evidence again.
