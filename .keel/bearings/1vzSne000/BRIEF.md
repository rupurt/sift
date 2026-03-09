# True Hybrid Retrieval Architecture — Brief

## Hypothesis

Sift should redefine `hybrid` to mean independent BM25 document retrieval plus
semantic vector retrieval over structure-aware sections, not BM25 retrieval plus
dense reranking of a BM25 shortlist. If sift represents long documents through
sections/pages/slides instead of one truncated whole-document embedding, it can
recover semantically relevant documents that lexical retrieval misses while
preserving the single-binary, no-daemon, no-database contract.

## Problem Space

The current implementation uses BM25 over the full corpus and then applies a
dense model only to the lexical shortlist. That improves ordering inside the
shortlist but it is not true hybrid retrieval, because a document that BM25
never surfaces cannot be recovered semantically.

This matters most on long and structured documents. HTML, PDF, and Office files
often contain relevant material in one section, page, slide, or worksheet while
the rest of the document is only weakly related to the query. A single
whole-document embedding or a rerank-only pipeline loses those retrieval
opportunities.

The architecture question is therefore two-fold:

1. What is the right vector-retrieval shape for sift under the current
   constraints?
2. Which runtime path keeps sift defensibly local-first and Rust-native?

## Success Criteria

How will we know if this research was valuable?

- [x] Determine whether sift should replace rerank-only `hybrid` behavior with
  true BM25 plus vector retrieval.
- [x] Recommend a document representation strategy that is effective for whole
  documents and structured rich formats.
- [x] Compare the current Candle path, `fastembed-rs`, and deferred alternatives
  against the repository constraints.
- [x] Produce a concrete epic/voyage recommendation that can be executed without
  reopening the architecture question.

## Open Questions

- Should the first true vector-retrieval slice use exact in-memory search over
  all section vectors or introduce an in-memory ANN structure immediately?
- Can the current Candle embedding path carry the first implementation, or is
  there enough benefit in `fastembed-rs` to justify a runtime tradeoff?
- How should sift aggregate section hits into a final document score and
  snippet?
