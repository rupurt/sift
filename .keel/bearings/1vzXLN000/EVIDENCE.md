---
id: 1vzXLN000
---

# Composable Search Strategy Architecture — Survey

## Current System Evidence

Code inspection of the current implementation shows:

- result fusion is Reciprocal Rank Fusion with `k = 60` in `src/hybrid.rs`;
- sift currently has no standalone reranking layer after fusion;
- the semantic channel is exact full-corpus segment retrieval aggregated to
  documents through a diminishing-returns rule in `src/vector.rs`;
- the current `hybrid` pipeline already runs BM25 and vector retrieval as
  independent channels in `src/search.rs`, but those channels are still wired
  through one monolithic engine path instead of a general strategy framework.

The completed benchmark voyage then showed the current exact no-index strategy
is operationally non-viable as the default champion:

- 3-query SciFact sample quality: hybrid underperformed BM25 on NDCG@10 and
  MRR@10, with recall unchanged.
- 3-query SciFact sample latency: hybrid measured roughly 129s p50 and 133s
  p90/max per query against a 200ms target.

That evidence matters, but it does not imply "drop vector retrieval." It implies
"stop treating one retrieval shape as the entire product strategy."

## Layered Strategy Model

The product direction that fits the new requirements is a layered strategy
pipeline:

1. query expansion,
2. parallel retrieval,
3. result fusion,
4. optional reranking.

This is also the right place to apply DDD and hexagonal boundaries:

- the domain should model search plans, query variants, candidate lists, fusion
  policies, rerank policies, and strategy presets;
- the application layer should orchestrate those domain steps;
- adapters should implement BM25, phrase/proximity retrieval, vector retrieval,
  benchmark sinks, CLI rendering, and model runtimes behind ports.

That structure makes sift easier to evolve and easier to benchmark honestly,
because each retriever and post-processing layer can be turned on, off, and
compared without rewriting the whole command path.

## Retrieval Techniques Beyond Vector Search

Vector retrieval is useful, but it is not the only or always best retrieval
technique for whole documents.

Useful first-class strategy families for sift are:

- BM25 document retrieval as the stable lexical baseline;
- phrase/proximity retrieval for ordered term windows and exact wording;
- structure-aware vector retrieval over sections/segments;
- query expansion for abbreviations, aliases, quoted phrases, and controlled
  rewrite variants;
- optional reranking over a fused shortlist.

Phrase matching deserves special attention because it covers a retrieval gap that
vector search does not solve cleanly:

- exact phrase retrieval is better when wording and term order matter;
- proximity retrieval is better when the query describes a tight concept but the
  exact phrase varies slightly;
- section-title or heading boosts are often more reliable than dense semantics
  for technical documents with strong structure.

This means the future `hybrid` champion should likely be a composition of
multiple lexical and semantic strategies, not "BM25 plus one vector channel and
done."

## PageIndex Applicability

PageIndex is useful to sift mainly as architectural guidance.

The PageIndex Framework describes a layered process where value-based search and
LLM tree search run in parallel, merge into a unique queue of nodes, and feed a
consumer agent. Its semantic document-search and hybrid tree-search tutorials
also emphasize a retrieval unit smaller than the final object: retrieve chunks,
aggregate to documents or nodes, and keep the parent object as the returned
unit.

What sift can borrow now:

- structure-aware sections/nodes as retrieval evidence,
- multiple retrievers executed in parallel,
- fusion of independent retrieval channels,
- optional reranking/consumer stages after retrieval,
- document- or node-level aggregation instead of returning raw chunks.

What should remain later follow-up work:

- iterative LLM-guided tree navigation,
- summary-generation loops,
- a full "consumer agent" runtime.

So yes, the proposed architecture can express a PageIndex-inspired preset in
sift. It cannot truthfully claim full PageIndex parity until sift grows a real
agentic navigation stage.

## Fusion And Reranking Options

RRF should remain the default fusion algorithm.

Reasons:

- sift already uses it successfully for independent ranked lists;
- it is robust when lexical and semantic scores are not calibrated;
- it keeps the fusion layer simple and explainable.

Reranking should become its own layer rather than being conflated with vector
search.

Recommended reranking progression:

1. `none` as the default;
2. optional dense reranker over a small fused shortlist;
3. optional LLM reranker over an even smaller shortlist for quality-focused
   workflows.

An LLM reranker is useful, but only as a bounded late-stage option. It is too
expensive and operationally variable to define the default retrieval path.

## Runtime Options

The current Candle path remains the best default for the first composable
strategy architecture because it preserves the local-first, pure-Rust story.

`fastembed-rs` remains interesting because it broadens model/runtime choices,
but its primary documented path still centers on ONNX Runtime and only exposes
limited Candle-backed model support in the official docs. That makes it a good
optional adapter candidate, not the first default.

## Benchmark Contract

The benchmark harness should explicitly model three comparison roles:

- `bm25`: the stable baseline every strategy is measured against;
- `champion`: the current best known named strategy preset;
- `candidate`: the strategy or preset under evaluation.

The top-level `hybrid` alias should resolve to `champion`, not to a permanently
hard-coded algorithm. That lets the product evolve while keeping the user-facing
entrypoint stable.

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | manual | manual:code-inspection | src/hybrid.rs | 2026-03-09 | 2026-03-09 | high | high | Current fusion algorithm is RRF with `k = 60` |
| SRC-02 | manual | manual:code-inspection | src/vector.rs | 2026-03-09 | 2026-03-09 | high | high | Semantic document scores aggregate segment hits with a diminishing-returns rule |
| SRC-03 | manual | manual:code-inspection | src/search.rs | 2026-03-09 | 2026-03-09 | high | high | Current `hybrid` path is a monolithic engine branch, not a generic strategy pipeline |
| SRC-04 | manual | manual:project-evidence | .keel/stories/1vzSwK000 | 2026-03-09 | 2026-03-09 | high | high | Completed benchmark story recorded BM25 vs hybrid quality and latency shortfall |
| SRC-05 | web | manual:web-search | https://pageindex.ai/blog/pageindex-intro | 2025-07-15 | 2026-03-09 | medium | medium | PageIndex framework describes parallel value-based search, LLM tree search, and a consumer agent |
| SRC-06 | web | manual:web-search | https://docs.pageindex.ai/tutorials/doc-search/semantics | 2025-08-10 | 2026-03-09 | medium | medium | PageIndex semantic doc search uses chunk retrieval aggregated back to documents |
| SRC-07 | web | manual:web-search | https://docs.pageindex.ai/tutorials/tree-search/hybrid | 2025-08-10 | 2026-03-09 | medium | medium | PageIndex hybrid tree search uses chunk retrieval aggregated back to nodes |
| SRC-08 | web | manual:github-review | https://github.com/VectifyAI/PageIndex | 2026-03-09 | 2026-03-09 | medium | high | Open-source PageIndex repository is Python-oriented rather than a drop-in Rust reuse path |
| SRC-09 | web | manual:web-search | https://docs.rs/fastembed/latest/fastembed/ | 2026-03-09 | 2026-03-09 | medium | high | fastembed docs emphasize ONNX runtime-backed embeddings and specific optional runtime support |

## Survey Conclusion

The next serious direction for sift is not a single retrieval rescue tactic. It
is a composable retrieval platform.

The recommended product shape is:

- DDD + hexagonal architecture,
- layered query expansion / retrieval / fusion / reranking,
- named strategy presets,
- `hybrid` as a configurable champion alias,
- BM25 as the required baseline,
- and PageIndex-inspired composition over structure-aware sections without yet
  claiming full agentic PageIndex behavior.
