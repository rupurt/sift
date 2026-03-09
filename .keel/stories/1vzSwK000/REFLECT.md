---
created_at: 2026-03-09T02:50:57
---

# Reflection - Benchmark True Hybrid Retrieval

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzXGL000: Title
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

- The benchmark harness needed two layers of proof: a fast fixture-based
  verification path for `keel verify run`, and a heavier SciFact sample for the
  actual evidence logs. Keeping those concerns separate let the story retain a
  reliable automated gate without pretending the real benchmark cost was small.
- Adding an explicit `--queries` path to `bench quality` was the smallest
  harness change that made sampled SciFact evaluation practical. It also makes
  the quality benchmark symmetric with the latency benchmark, which already
  accepted explicit queries.
- The measured shortfall is severe. On the 3-query SciFact sample, the current
  exact no-index hybrid path reported p50 latency around 129 seconds and p90
  around 133 seconds per query, while hybrid quality was also worse than BM25
  on NDCG and MRR in that sample. That is strong evidence that the current
  architecture is functionally correct but not yet performance-viable for the
  original target without further design changes.
