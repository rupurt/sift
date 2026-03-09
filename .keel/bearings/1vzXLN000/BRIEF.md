# Composable Search Strategy Architecture — Brief

## Hypothesis

Sift should stop treating search as one hard-coded engine branch and instead
model search as a layered, composable strategy pipeline:

1. query expansion,
2. parallel retrieval,
3. result fusion,
4. optional reranking.

Under that model, `bm25` remains the stable baseline strategy, multiple named
hybrid presets become first-class, and `hybrid` becomes a configurable alias to
the current champion preset. This architecture can absorb BM25, phrase and
proximity retrieval, structure-aware vector retrieval, optional dense or LLM
reranking, and PageIndex-inspired composite strategies without pretending that
vector search is the only or best answer.

## Problem Space

The completed true-hybrid voyage proved two things at once:

- independent BM25 + vector retrieval over structure-aware segments is a real
  retrieval shape that fits sift's thesis;
- a no-index exact full-corpus vector path is too slow to be the obvious
  champion strategy.

That evidence makes the broader product question unavoidable. The next problem
is not just "make the current hybrid engine faster." The next problem is "what
is the right retrieval platform for sift?"

Right now sift still lacks first-class architecture for:

- query expansion,
- parallel lexical and semantic search,
- multiple named hybrid strategies,
- fusion and reranking as independent layers,
- fair benchmarking against both a stable baseline and a moving best-known
  strategy.

The product direction now also includes a maintainability constraint: the search
domain should be structured with DDD and hexagonal boundaries so new strategies
do not keep leaking across CLI, benchmark, and runtime code paths.

## Success Criteria

This research is valuable if it turns that product-direction question into a
board-ready architecture decision.

- [ ] Define the layered search architecture sift should use going forward:
      query expansion, retrieval, fusion, and reranking.
- [ ] Clarify the current fusion and reranking behavior and recommend the next
      default algorithms.
- [ ] Compare vector search with other useful retrieval techniques such as
      phrase and proximity retrieval rather than treating vector search as the
      only answer.
- [ ] Determine what PageIndex ideas fit sift now, and what remains a later
      agentic/tree-search follow-up.
- [ ] Produce a concrete epic/voyage recommendation that makes BM25 the stable
      baseline and benchmarks every candidate strategy against both BM25 and
      the current champion preset.

## Open Questions

- Should the first reranking abstraction ship only with `none`, or should it
  also expose a concrete dense reranker immediately?
- Should a future LLM reranker be local-only, remote-optional, or explicitly
  deferred until the strategy architecture is stable?
- How far should the first PageIndex-inspired preset go before the product
  truly needs LLM-guided tree navigation?
- Does the champion preset live in static code configuration first, or does it
  need a richer user-configurable registry immediately?
