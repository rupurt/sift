---
created_at: 2026-03-10T16:48:28
---

# Reflection - Add Sift-Embed Example Crate And Just Recipes

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VDVlspyQD: Title
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

- A standalone consumer crate nested under `examples/` needs its own empty `[workspace]` table when the repo root is not converted into a Cargo workspace; otherwise Cargo reports that the package "believes it's in a workspace when it's not" for commands scoped to the example manifest.
- Keeping the example on the supported embedding path was straightforward once it rendered `SearchResponse` locally instead of reaching for existing CLI-only helpers. That preserves the crate-root facade as the only public contract the example demonstrates.
- `cargo check --manifest-path examples/sift-embed/Cargo.toml` was not a reliable acceptance proof in this environment because the existing `mold` and glibc linker failure still trips transitive build-script link steps. Manifest and source inspection via `cargo metadata` plus targeted `rg` checks gave a stable proof path for this slice.
