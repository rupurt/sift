# Recover Hybrid Retrieval Viability — Survey

## Evidence From The Completed Voyage

- The true-hybrid voyage completed successfully at the implementation level:
  structure-aware segments, full-corpus semantic retrieval, RRF fusion, and
  best-segment snippets all work in the local CLI.
- The benchmark story recorded explicit sample evidence on SciFact:
  - 3-query sample quality: hybrid underperformed BM25 on NDCG@10 and MRR@10,
    with recall unchanged.
  - 3-query sample latency: hybrid measured roughly 129s p50 and 133s p90/max
    per query against a 200ms target.
  - A 25-query sample and an uncapped 301-query run were both operationally too
    slow for routine execution, reinforcing that the shortfall is architectural
    rather than cosmetic.

## Constraints That Still Matter

- Keep a single Rust binary.
- No external database.
- No daemon/background service.
- Local model execution should remain pure-Rust by default.

These constraints still appear compatible with the product thesis. The
constraint that now needs reconsideration is the earlier avoidance of persisted
local sidecar indexes.

## Candidate Recovery Directions

### 1. Persisted Local Embedding Sidecar

Store chunk/segment embeddings and minimal retrieval metadata locally on disk,
then load/query them from the CLI on demand.

Pros:
- strongest path to sub-second hybrid retrieval;
- preserves single-binary/local UX;
- avoids recomputing embeddings for every query.

Cons:
- explicit departure from the earlier "no persisted sidecar index" preference;
- requires invalidation/freshness strategy.

### 2. In-Memory ANN Or Session Index

Build an ANN structure per invocation or for a batch session without persisting
it to disk.

Pros:
- preserves the no-sidecar stance;
- simpler product story than persisted state.

Cons:
- still pays heavy preparation cost each run;
- unlikely to recover enough end-to-end latency for one-shot CLI usage.

### 3. Keep Exact Search But Tune Model/Segmentation

Attempt better chunking, richer segment boundaries, or alternative models.

Pros:
- smallest conceptual change.

Cons:
- the measured latency miss is so large that tuning alone is unlikely to be
  enough;
- quality also regressed on the sample, so the risk is high.

## Survey Conclusion

The completed voyage produced enough evidence to justify planning a persisted
local sidecar index or equivalent precomputed vector store as the next serious
recovery direction. Pure query-time exact semantic retrieval over the full
active corpus is now proven too expensive for the target product behavior.
