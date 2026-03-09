---
id: 1vzSne000
---

# True Hybrid Retrieval Architecture — Survey

## Market Research

### Existing Solutions

Hybrid retrieval systems generally split into two forms:

- Lexical retrieval plus semantic reranking of a lexical shortlist.
- Independent lexical retrieval plus independent vector retrieval with a fusion
  stage.

Sift currently implements the first form. The product request now points toward
the second form: BM25 should remain the lexical channel, but the semantic path
should become its own retriever.

The most relevant external prior art here is PageIndex. Its semantic document
search tutorial recommends chunking documents, embedding the chunks, searching
for top-K chunks, and then computing a document score from the retrieved chunks
instead of returning the chunks directly. Its hybrid tree-search tutorial uses
the same idea for tree nodes: search chunks, aggregate chunk scores to a node
score, and retrieve the parent node rather than the raw chunk. That is a strong
fit for sift's "return documents, not fragments" CLI contract.

### Competitive Landscape

The design space for sift is different from hosted vector stacks:

- Full vector databases solve this with persisted embeddings and ANN indexes,
  but that conflicts with the current no-database / no-sidecar-index posture.
- Rerank-only CLIs are simpler and faster, but they do not satisfy the intended
  meaning of hybrid retrieval.
- Structured retrieval frameworks such as PageIndex show that section-backed
  document retrieval is the right abstraction for long documents.

### Opportunity

The opportunity is not another generic vector stack. The opportunity is a local
CLI that:

- works on raw local corpora,
- treats long documents through structure-aware sections,
- and combines lexical precision with semantic recovery without introducing a
  resident service.

## Technical Research

### Option 1: Keep Candle And Add True Vector Retrieval

This option reuses sift's current local embedding path and changes only the
retrieval shape:

1. segment each document into structure-aware sections,
2. embed all active sections for the query,
3. run exact similarity search over section vectors,
4. aggregate section scores into document scores,
5. fuse BM25 document ranks and vector-derived document ranks.

Why this fits:

- It preserves the current pure-Rust, local-first inference path.
- It does not require ONNX Runtime or native dynamic-linking behavior.
- It keeps the first implementation simple: exact search over an in-memory
  vector list is enough for the current "thousands of files" scale.

Main cost:

- The expensive part is embedding all sections for a query-time corpus. Exact
  similarity search itself is cheap once vectors exist.

### Option 2: Adopt `fastembed-rs`

`fastembed-rs` is appealing because it exposes many embedding models behind a
clean Rust API, including `all-MiniLM-L6-v2`. However, its README says the
library uses `ort` for ONNX inference by default, and the Candle-backed support
is called out only for specific models such as Qwen3 embeddings and
`nomic-embed-text-v2-moe`.

Implications for sift:

- `fastembed-rs` is a real option if we decide that ONNX Runtime is acceptable.
- It is not the cleanest default if we want the primary runtime story to remain
  pure Rust.
- It is better treated as a deferred or optional runtime path than as the first
  implementation choice.

The `ort` documentation also shows that the default feature set downloads
prebuilt binaries and manages dynamic libraries, which increases packaging and
linking complexity relative to the current Candle path.

### Option 3: Add ANN Immediately

An in-memory HNSW or similar ANN structure would reduce vector-search cost once
embeddings exist, but it does not solve the dominant cost in the current
architecture: per-query embedding of the active corpus sections.

Given the repository constraints, ANN is therefore a second-order optimization:

- useful once repeated-query workflows exist,
- but not required for the first correct version of true hybrid search.

### Option 4: Whole-Document Vectors

This is the simplest semantic retrieval design, but it is the wrong one for
long documents.

Problems:

- a single truncated embedding misses relevant local sections,
- structured sources such as HTML, PDF, and slides lose their internal shape,
- snippets become harder to justify because the retriever never tracked the best
  local evidence.

### Structure-Aware Sections

This is the key architectural insight from the research.

For sift, the vector retrieval unit should be a section-like segment:

- plain text: heading blocks or paragraph windows,
- HTML: heading/section blocks,
- PDF: page-backed sections first, richer structure later,
- PowerPoint: slide-backed sections,
- Excel: sheet/table-backed sections,
- Word: heading/paragraph sections.

Those segments provide:

- better semantic recall on long documents,
- explainable snippets,
- and a stable bridge from vector hits back to document-level results.

### Aggregation And Fusion

PageIndex's semantic document search and hybrid tree-search tutorials both use a
chunk-to-parent aggregation rule with diminishing returns: sum chunk scores and
divide by `sqrt(N + 1)` for `N` retrieved chunks. The important design property
is not the exact formula; it is the principle that multiple relevant sections
should help a document, but not allow large documents to dominate purely by
size.

For lexical + vector fusion, rank-based fusion is a better fit than the current
weighted score blending. Qdrant's hybrid-search documentation treats Reciprocal
Rank Fusion (RRF) as the standard way to combine independently retrieved sparse
and dense result sets, and it explicitly distinguishes fusion from reranking.

For sift, the practical takeaway is:

- aggregate section hits into document-level vector ranks,
- then fuse BM25 document ranks with vector document ranks via RRF.

## User Research

### Target Users

- Developers searching local technical corpora from the terminal.
- Coding agents that need document-level retrieval grounded in local files.

### Pain Points

- The current `hybrid` engine cannot recover semantically relevant documents
  that BM25 fails to shortlist.
- Long documents are underserved by truncation-heavy dense scoring.
- Rich formats need a retrieval unit that preserves local structure instead of
  flattening everything into one opaque document vector.

### Validation

The user request itself sharpens the product contract: "hybrid" should mean
BM25 plus vector retrieval. The desire for structure-aware sections also aligns
with the most credible external prior art rather than being a cosmetic
preference.

## Key Findings

1. Sift's current `hybrid` engine is a rerank pipeline, not true hybrid
   retrieval.
2. True hybrid retrieval should replace the dense rerank stage inside `hybrid`
   with independent vector retrieval over structure-aware sections.
3. PageIndex provides strong architectural guidance for sift even though its
   implementation model is not directly reusable: retrieve chunks, aggregate to
   parent docs/nodes, and return the parent object.
4. The first true-hybrid slice should use exact in-memory vector search over
   section embeddings rather than ANN or persisted indices.
5. Candle remains the best first runtime for sift's default path; `fastembed-rs`
   is promising but is not a general Candle-backed replacement for the current
   embedder.

## Unknowns

- Whether the first exact-search implementation can remain within the 200 ms
  target on realistic rich-document corpora.
- How much extractor work is needed to emit high-quality section boundaries for
  each supported format.
- Whether a future optional rerank stage is still worth keeping after true
  hybrid retrieval lands.

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | web | manual:web-search | https://docs.pageindex.ai/tutorials/doc-search/semantics | 2026-03-08 | 2026-03-08 | medium | high | PageIndex semantic document search tutorial recommends chunk-to-document aggregation |
| SRC-02 | web | manual:web-search | https://docs.pageindex.ai/tutorials/tree-search/hybrid | 2026-03-08 | 2026-03-08 | medium | high | PageIndex hybrid tree search tutorial uses chunk-to-node score aggregation |
| SRC-03 | web | manual:github-review | https://github.com/Anush008/fastembed-rs | 2026-03-08 | 2026-03-08 | medium | high | fastembed-rs README documents ort-backed ONNX inference and limited Candle support |
| SRC-04 | web | manual:web-search | https://ort.pyke.io/setup/cargo-features | 2026-03-08 | 2026-03-08 | medium | high | ort cargo feature docs show prebuilt binary downloads and dynamic library management |
| SRC-05 | web | manual:web-search | https://qdrant.tech/documentation/concepts/hybrid-queries/ | 2026-03-08 | 2026-03-08 | high | high | Qdrant hybrid-query docs treat RRF as standard sparse+dense fusion method |
