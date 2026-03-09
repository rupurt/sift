---
created_at: 2026-03-09T01:22:31
---

# Reflection - Add Full-Corpus Segment Vector Retrieval

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzVsl000: Title
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

- Pulling the semantic path into its own `vector` module kept this story
  reusable. The dense model now acts as a scorer over segments, while
  aggregation and ranking logic stay independent of the embedding backend.
- Computing the query embedding once and then batching segment embeddings was
  the smallest viable step from reranking to full-corpus search. That preserves
  the existing Candle path while avoiding a monolithic all-segments batch.
- The story deliberately switches `hybrid` away from BM25-shortlist reranking
  before RRF fusion exists. That makes the semantic path testable in isolation,
  but the next story needs to restore the lexical signal explicitly rather than
  assuming the old shortlist behavior still exists.
