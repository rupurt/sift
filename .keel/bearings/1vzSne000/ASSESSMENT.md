---
id: 1vzSne000
---

# True Hybrid Retrieval Architecture — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | This fixes a product-contract gap in the meaning of `hybrid` and improves retrieval on long documents. |
| Confidence | 4 | The architectural direction is well-supported by current code constraints and external prior art. |
| Effort | 5 | The work spans document segmentation, vector retrieval, aggregation, fusion, and benchmark evidence. |
| Risk | 4 | The largest risk is latency after moving from reranking to corpus-wide vector retrieval. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Opportunity Cost

Pursuing true hybrid retrieval now delays secondary improvements such as ANN
optimizations, richer snippet UX, or a future dedicated rerank stage. That is
acceptable because the current hybrid engine does not satisfy the intended
retrieval contract and would otherwise force later work to build on a misleading
foundation.

### Findings

- PageIndex tutorials demonstrate chunk-to-parent aggregation as the standard pattern for document-level retrieval from segment hits [SRC-01] [SRC-02]
- fastembed-rs relies on ONNX Runtime by default, making Candle the better first runtime for pure-Rust constraints [SRC-03] [SRC-04]
- RRF is the standard fusion method for combining independently retrieved sparse and dense result sets [SRC-05]

### Dependencies

The following need to hold:

- sift needs a structure-aware segment abstraction beneath document ranking [SRC-01] [SRC-02]
- the current Candle embedding path must remain viable as the default local
  runtime for the first implementation slice [SRC-03] [SRC-04]
- benchmark work must explicitly measure both retrieval-quality gains and any
  latency regressions introduced by whole-corpus vector retrieval [SRC-05]

### Alternatives Considered

Alternatives considered:

- Keep rerank-only hybrid semantics and rename the feature.
  Rejected because the user objective is true BM25 plus vector retrieval, not a
  documentation fix. [SRC-05]
- Replace the current embedding path with `fastembed-rs` immediately.
  Deferred because the default documented path relies on ONNX Runtime, while
  Candle already satisfies the repository's preferred runtime constraints. [SRC-03] [SRC-04]
- Use single whole-document embeddings.
  Rejected because it is weak for long and structured documents and does not
  exploit the section-aware insight from PageIndex. [SRC-01] [SRC-02]
- Introduce persisted vector sidecars immediately.
  Deferred because the current board still prefers transient or ephemeral
  structures first and requires evidence before relaxing that constraint. [SRC-05]

## Recommendation

[x] Proceed → convert to epic [SRC-01] [SRC-05]
[ ] Park → revisit later
[ ] Decline → document learnings

Proceed with a new epic for true hybrid structure-aware retrieval.

Recommended sequence:

1. Introduce a structure-aware segment model beneath search.
2. Implement exact in-memory vector retrieval over all active segments.
3. Aggregate segment hits into document-level vector scores using a
   PageIndex-inspired diminishing-returns formula.
4. Replace the current hybrid fusion path with BM25 document retrieval plus
   vector document retrieval fused through RRF.
5. Re-run quality and latency benchmarks with explicit evidence for the new
   architecture.

Deferred follow-ups:

- in-memory ANN acceleration,
- optional alternative embedding runtimes such as `fastembed-rs`,
- optional post-fusion reranking,
- persisted vector sidecars if benchmark evidence proves they are necessary.
