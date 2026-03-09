# Recover Hybrid Retrieval Viability — Brief

## Hypothesis

The current structure-aware true-hybrid design is functionally correct but will
not reach the product latency/quality targets without changing the retrieval
architecture. The benchmark evidence likely justifies planning an explicit
indexing or precomputation step that was previously out of scope.

## Problem Space

The completed true-hybrid voyage replaced shortlist reranking with BM25 plus
full-corpus vector retrieval over structure-aware segments. The resulting CLI
works end to end, but the recorded SciFact sample evidence shows two critical
problems:

- hybrid latency is orders of magnitude above the 200 ms target;
- hybrid quality did not beat BM25 on the sampled evaluation slice.

The repo now needs a recovery direction that preserves the local single-binary
product thesis while addressing the proven performance and quality gap.

## Success Criteria

This research is valuable if it turns the benchmark shortfall into a concrete,
board-ready architecture decision.

- [ ] Identify which current constraints are now proven too expensive in
      practice and which still need to hold.
- [ ] Recommend the next recovery direction with explicit tradeoffs across
      latency, quality, complexity, and product constraints.

## Open Questions

- Is a persisted local embedding sidecar now justified by the benchmark data?
- Would an in-memory ANN/session index recover enough performance without
  violating the current UX goals?
- Is the quality regression primarily a retrieval-architecture issue, a
  segmentation issue, or a model/chunking issue?
