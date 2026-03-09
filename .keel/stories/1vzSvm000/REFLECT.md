---
created_at: 2026-03-09T01:14:02
---

# Reflection - Add Structure-Aware Segment Model

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzVkY000: Title
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

- Keeping BM25 indexing on the original document text made this story low-risk.
  The new segment model could be added beneath `Document` without changing the
  current ranking behavior, which kept the acceptance criteria focused on
  structure rather than retrieval regressions.
- The extractor surface still returns normalized text only. Source-aware
  segmentation for PDF, PPTX, and XLSX therefore relies on deterministic text
  heuristics today instead of true page/slide/sheet metadata. That is good
  enough for this story, but the next vector-retrieval slice should treat the
  segment builder as the place where richer structural signals can plug in.
- Mixed-format fixture coverage was enough to catch the key contract: stable
  segment IDs, at least one segment per supported document, and preservation of
  local section text. That gives the next story a concrete corpus-level API to
  build semantic retrieval on top of.
