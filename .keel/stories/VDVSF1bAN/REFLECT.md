---
created_at: 2026-03-10T15:57:53
---

# Reflection - Adopt The Facade In The Executable

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VDVZ99rMy: Title
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

The executable cutover was smaller than the original direct `run_search`
assembly suggested. Once `SearchCommand` translated its clap state into
`SearchInput` and `SearchOptions`, the CLI only needed to own argument parsing
and output rendering while the facade handled planning, embedder selection, and
request execution.

Keeping clap-facing enums local to `src/main.rs` is the right boundary here.
That preserves the exact CLI tokens while avoiding a new dependency from the
supported library surface back to clap-derived internal enums.

`cargo check --bin sift` passed after the cutover, but repo-level runnable proof
is still blocked in this environment. `cargo run -- --help`, `cargo run -- search
--help`, and `nix develop -c just check` all hit the same linker failure under
`mold` with unresolved symbols including `open64`, `lseek64`,
`gnu_get_libc_version`, and related glibc-era entry points.
