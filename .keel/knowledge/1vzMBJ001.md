---
source_type: Story
source: stories/1vzJfv000/REFLECT.md
scope: 1vzJVa000/1vzJda000
source_story_id: 1vzJfv000
created_at: 2026-03-08T15:21:14
---

### 1vzMBJ001: Keep Keel Verify Proofs Bounded

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Stories whose acceptance evidence depends on long-running full-corpus benchmark commands |
| **Insight** | `keel verify run` is better used for bounded smoke proofs and targeted tests; full release benchmarks are more reliable when recorded separately with `keel story record --cmd`, which preserves the exact command output in evidence logs without making the verifier brittle. |
| **Suggested Action** | Put the exact long benchmark commands in `story record` evidence, and keep README `verify:` annotations short enough to pass consistently under the verifier harness. |
| **Applies To** | `.keel/stories/*/README.md`, benchmark-heavy stories |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-08T22:24:00+00:00 |
| **Score** | 0.76 |
| **Confidence** | 0.89 |
| **Applied** | yes |
