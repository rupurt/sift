---
created_at: 2026-03-10T21:51:54
---

# Reflection - Cut 0.2.0 Release Candidate

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VDX0GvBKW: Title
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

- The version cut itself was straightforward once the repository and embedded
  example crate were updated together; leaving the example at `0.1.0` would
  have created an avoidable stale-version pocket inside the repo.
- The real release blocker was cargo-dist's generated-CI check. Because the
  workflow is intentionally hand-customized to inject `SIFT_GIT_SHA`, the
  repository needs `allow-dirty = ["ci"]` in `workspace.metadata.dist` for
  cargo-dist planning to accept the workflow as authoritative.
- Story proof commands that invoke Nix-backed tooling need to `cd` to the repo
  root explicitly. `keel story record` can execute from inside `.keel`, which
  is enough to break `cargo dist` unless the command pins the working
  directory first.
