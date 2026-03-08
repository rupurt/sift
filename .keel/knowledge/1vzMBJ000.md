---
source_type: Story
source: stories/1vzJfv000/REFLECT.md
scope: 1vzJVa000/1vzJda000
source_story_id: 1vzJfv000
created_at: 2026-03-08T15:21:14
---

### 1vzMBJ000: Sequence Length Dominates Hybrid Reranker Latency

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Tuning Candle-based dense reranking for transient hybrid search over thousands of raw documents |
| **Insight** | On the SciFact workload, lowering the dense encoder `max_length` had a much larger latency effect than reducing the BM25 shortlist. `all-MiniLM-L6-v2` at `max_length 40` kept a material quality gain over BM25 while bringing full-run hybrid latency back under the p50/p90 target. |
| **Suggested Action** | Treat sequence length as the primary performance dial before weakening shortlist depth or replacing the model, and record both quality and full-query latency before changing defaults. |
| **Applies To** | `src/dense.rs`, `src/main.rs`, `src/bench.rs`, hybrid search benchmarks |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-08T22:22:00+00:00 |
| **Score** | 0.94 |
| **Confidence** | 0.93 |
| **Applied** | yes |
