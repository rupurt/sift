---
created_at: 2026-03-10T16:53:07
---

# Reflection - Document The Runnable Embedded Example

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VDVn3UmhO: Title
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

- The root README already had the library facade documented, so the missing piece was not API explanation but a concrete handoff to a runnable consumer. Adding one canonical pointer kept the docs focused instead of duplicating the entire embedding section in multiple places.
- A colocated `examples/sift-embed/README.md` works well for this kind of slice because it keeps the exact `sift-embed search [OPTIONS] [PATH] <QUERY>` contract beside the example crate itself while the repo README only needs the high-signal entry path.
- Repo-wide verification is still constrained by the existing `mold` and glibc linker failure in `just check`, so the acceptance proof for this docs-only story had to stay grounded in executable `rg` checks against the authored documentation instead of broader cargo-based validation.
