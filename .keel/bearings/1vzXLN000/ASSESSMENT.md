---
id: 1vzXLN000
---

# Composable Search Strategy Architecture — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | This changes sift from one hard-coded retrieval engine into a platform for evolving and benchmarking search strategies. |
| Confidence | 4 | The direction is supported by current code evidence, the observed benchmark shortfall, and PageIndex's layered retrieval model. |
| Effort | 4 | The first voyage is architectural and cross-cutting, but it can reuse existing BM25, segment, and vector capabilities. |
| Risk | 3 | The main risk is over-scoping PageIndex or reranking too early rather than building clean composition boundaries first. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Opportunity Cost

The main opportunity cost is not immediately pursuing one specific latency fix
such as persisted vector sidecars. That is acceptable because the broader
product need is now clearer: sift requires a composable strategy architecture
before it can evaluate which retrieval and reranking tactics actually deserve to
be the default champion.

### Findings

- sift already uses RRF for fusion, but it has no standalone reranking layer yet
  and no general strategy abstraction [SRC-01] [SRC-03]
- the benchmark shortfall proves the current exact vector strategy should not be
  frozen as the permanent `hybrid` definition [SRC-04]
- PageIndex validates the layered direction: parallel search, structure-aware
  evidence, and aggregation from chunks to parent units [SRC-05] [SRC-06] [SRC-07]
- vector retrieval is one useful strategy, not the whole answer; phrase and
  proximity retrieval deserve first-class treatment alongside it [SRC-03] [SRC-05]
- `fastembed-rs` is viable as a future adapter but not the best first default
  while Candle remains the cleanest pure-Rust path [SRC-09]

### Dependencies

The following must hold:

- sift needs a domain model for search plans, strategy presets, and candidate
  flows that sits above concrete retrievers and runtimes [SRC-03]
- the first voyage needs clean ports for query expansion, retrieval, fusion,
  reranking, and benchmark comparison so later algorithms remain swappable
  [SRC-05] [SRC-07]
- BM25 must remain a stable benchmark baseline and `hybrid` must become a
  configurable champion alias rather than a permanently hard-coded algorithm
  [SRC-04]

### Alternatives Considered

Alternatives considered:

- Keep optimizing the current monolithic hybrid implementation in place.
  Rejected because it still leaves sift without a general model for composing or
  benchmarking alternative strategies. [SRC-03] [SRC-04]
- Make vector retrieval the sole semantic answer inside `hybrid`.
  Rejected because phrase/proximity retrieval and query expansion address
  different failure modes that vectors do not fully cover. [SRC-05] [SRC-06]
- Jump straight to full PageIndex-style agentic tree search.
  Deferred because sift does not yet have the intermediate composition
  boundaries, node catalog, or consumer-agent runtime to support it cleanly.
  [SRC-05] [SRC-07] [SRC-08]
- Make an LLM reranker the default final layer.
  Rejected because reranking should first become a bounded optional stage; cost,
  latency, and runtime-policy choices should not be coupled to the baseline
  architecture. [SRC-03] [SRC-05]

## Recommendation

[x] Proceed → convert to epic [SRC-01] [SRC-05]
[ ] Park → revisit later
[ ] Decline → document learnings

Proceed with a composable search-strategy epic.

Recommended sequence:

1. Formalize DDD and hexagonal boundaries for search plans, query expansion,
   retrieval, fusion, reranking, and benchmark comparison.
2. Make BM25 the explicit baseline strategy and promote `hybrid` into a
   configurable champion alias.
3. Add phrase/proximity retrieval and vector retrieval as parallel retriever
   options rather than hiding them inside one engine branch.
4. Keep RRF as the default fusion policy and introduce a reranker port with
   `none` as the default implementation.
5. Ship named strategy presets, including a PageIndex-inspired preset composed
   from available layers.
6. Benchmark every preset against both BM25 and the current champion.

Deferred follow-ups:

- concrete LLM reranker adapters,
- full PageIndex-style agentic tree navigation,
- persisted vector sidecars or ANN acceleration if the benchmark champion still
  cannot meet target latency.
