---
id: 1vzSne000
---

# True Hybrid Retrieval Architecture — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | This is a fundamental change to Sift's search quality, replacing a weaker rerank-only pipeline with true hybrid retrieval [SRC-01]. |
| Confidence | 4 | PageIndex and Qdrant have proven this model, and the component parts are well-understood [SRC-01] [SRC-02] [SRC-05]. |
| Effort | 3 | Requires refactoring the core search path, but reuses existing BM25 and Candle embedding logic [SRC-03]. |
| Risk | 3 | Latency is the primary risk; embedding all sections for every query may be too slow without caching [SRC-01] [SRC-03]. |

*Scores range from 1-5 (1=Very Low, 5=Very High)*

## Analysis

### Overview
Deferring this means Sift's "hybrid" strategy remains misleading and semantically weak.

## Opportunity Cost

Deferring this means Sift's "hybrid" strategy remains misleading and semantically weak. It also blocks progress on structure-aware search for rich documents [SRC-01]

## Findings

- True hybrid retrieval (independent channels + fusion) is the industry standard for combining lexical and semantic signals [SRC-01] [SRC-05]
- Structure-aware section retrieval is essential for long documents, as validated by PageIndex [SRC-01] [SRC-02]
- Reciprocal Rank Fusion (RRF) is the best choice for combining uncalibrated scores from BM25 and vector retrievers [SRC-05]

## Dependencies

- The core domain must support independent retrieval channels and a fusion stage [SRC-05]
- The document model must be extended to include structure-aware `Segment`s [SRC-01]

## Alternatives Considered

- **Keep Rerank-Only:** Continue with the existing architecture. Rejected because it fails to recover semantically relevant documents that don't appear in the lexical shortlist [SRC-01]
- **Whole-Document Vectors:** Simpler, but proven to be less effective for long, structured documents where local context is key [SRC-01] [SRC-02]
- **Use `fastembed-rs`:** Deferred because it introduces an ONNX Runtime dependency, which conflicts with the pure-Rust goal for the default path [SRC-03] [SRC-04]

## Recommendation

- [x] Proceed → convert to epic [SRC-01] [SRC-05]
- [ ] Park → revisit later
- [ ] Decline → document learnings

Proceed with an Epic to implement "True Hybrid Retrieval." The first voyage should replace the current rerank stage with an independent vector retrieval channel over structure-aware document sections, using RRF for fusion [SRC-01] [SRC-05]
