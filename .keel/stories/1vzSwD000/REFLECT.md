---
created_at: 2026-03-09T01:27:55
---

# Reflection - Fuse BM25 And Vector Retrieval In Hybrid Search

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzVxz000: Title
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

- RRF was a clean cutover because the lexical and semantic paths were already
  separated by the prior story. The fusion layer only needed document IDs,
  ranks, and semantic snippet provenance; it did not need to know anything
  about the embedding runtime.
- Carrying snippet text on the ranked result model was the smallest practical
  way to make hybrid rendering evidence-aware. That avoided forcing BM25 and
  hybrid paths to share one snippet strategy while still letting the hybrid path
  prefer best-segment text deterministically.
- The fixture corpus still includes benchmark TSV files when searching the
  whole `tests/fixtures/rich-docs` tree. That does not break the fusion proof,
  but it is a reminder that raw `search` currently treats any readable UTF-8
  file as searchable input unless explicitly filtered elsewhere.
