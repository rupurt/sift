---
id: 1vzJVa000
---

# Raw Document Retrieval Architecture Research — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | This is the product-defining architecture for the repository and replaces an invalid `zvec`-centric thesis. |
| Confidence | 4 | The sparse path is straightforward, and pure-Rust dense inference is feasible, but the latency target still needs proof. |
| Effort | 4 | Delivering corpus tooling, benchmark harnesses, BM25 retrieval, and dense reranking is meaningful but bounded MVP work. |
| Risk | 3 | The main risk is missing the 200 ms target on CPU without a persisted index. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Opportunity Cost

The main opportunity cost is spending engineering time on benchmark and eval
infrastructure before feature breadth such as PDF, HTML, and Office ingestion.
That trade is correct because the retrieval architecture is a prerequisite for
all later format support.

### Findings

- The bm25 crate provides a viable in-memory BM25 search engine for transient indexing [SRC-01]
- Candle supports BERT and JinaBert sentence-embedding workloads for pure-Rust local inference [SRC-02]
- The all-MiniLM-L6-v2 model is small enough (22.7M parameters, 384-dim) for CPU-first local embedding [SRC-05]
- BEIR SciFact provides a standard corpus/queries/qrels layout suitable for first benchmark cycle [SRC-06]

### Dependencies

The following must hold:

- the CLI can materialize or ingest an evaluation corpus locally [SRC-06]
- BM25 retrieval over raw files can be implemented without hidden persistent
  state [SRC-01]
- a pure-Rust embedding path can load and run a small sentence-transformer
  model on CPU [SRC-02]
- benchmark commands can be made reproducible and attached as board evidence [SRC-06]

### Alternatives Considered

Alternatives considered:

- Keep pursuing `zvec` and disk-backed index files.
  Rejected because it conflicts with the current operating contract. [SRC-01]
- Ship BM25-only first and defer hybrid as optional.
  Rejected because hybrid ranking is a non-negotiable default requirement. [SRC-02]
- Compute dense scores across the full corpus on every query.
  Rejected as the least likely path to the 200 ms target without persistence. [SRC-05]
- Start with Burn-first ONNX import.
  Deferred. Burn is viable, but direct Candle integration is a faster MVP
  route and still satisfies the pure-Rust runtime constraint. [SRC-03] [SRC-04]

## Recommendation

- [x] Proceed → convert to epic [SRC-01] [SRC-02] [SRC-05]
- [ ] Park → revisit later
- [ ] Decline → document learnings

Proceed with a new epic that replaces the stale `zvec`-oriented product thesis
and decomposes the work into:

1. evaluation corpus and benchmark harness
2. raw-document BM25 baseline
3. pure-Rust dense encoder plus hybrid fusion
4. CLI UX and richer format expansion after MVP
