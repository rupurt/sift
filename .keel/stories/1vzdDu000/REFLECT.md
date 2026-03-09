# Reflect - Add Fusion And Reranking Layers (1vzdDu000)

## What was learned?
- Separating fusion from retrieval allows for a clean M:N mapping where multiple retrievers can be combined using a single fusion policy.
- Preserving provenance in the `Candidate` model is essential for explainability in hybrid search.
- The `Reranker` port provides a clear boundary for future high-precision stages like Cross-Encoders or LLM-based reranking.

## Any surprises?
- RRF implementation is straightforward but its effectiveness depends heavily on the relative quality of the input rankings.

## Future improvements?
- Implement a `WeightedSum` fuser for cases where retriever quality is known and stable.
- Add a `CrossEncoderReranker` adapter.
