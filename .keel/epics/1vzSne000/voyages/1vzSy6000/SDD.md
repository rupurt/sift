# Structure-Aware True Hybrid Retrieval - Software Design Description

> Redefine sift's hybrid engine as BM25 document retrieval plus vector retrieval over structure-aware sections, with chunk-to-document aggregation and rank-based fusion that preserve the single-binary, no-daemon, no-database contract.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage changes hybrid retrieval from "BM25 over the corpus, then dense
rerank the lexical shortlist" to a dual-retrieval system:

1. BM25 retrieves documents over the active corpus.
2. Vector retrieval searches structure-aware segments across the same corpus.
3. Segment hits aggregate into document-level semantic ranks.
4. BM25 and vector document ranks are fused into the final result set.

The voyage deliberately keeps the first implementation simple and local-first:
exact in-memory vector search over section embeddings, no ANN requirement, and
no persisted vector index.

## Context & Boundaries

```
┌───────────────────────────────────────────────────────────────────┐
│                            This Voyage                            │
│                                                                   │
│  ExtractedDocument -> Segment Builder -> Vector Search -> Doc Agg │
│         |                    |                  |          |       │
│         +--------------------+                  |          |       │
│                                                 v          |       │
│                         BM25 Document Retrieval ----------> RRF    │
│                                                           |       │
│                                                           v       │
│                                             Document Results +    │
│                                             best-section snippet  │
└───────────────────────────────────────────────────────────────────┘
          ↑                             ↑                     ↑
    local file corpus             local model cache      bench/eval CLI
```

### In Scope

- structure-aware segmentation,
- full-corpus vector retrieval over active segments,
- document aggregation and RRF fusion,
- benchmark/eval updates for the true-hybrid path.

### Out of Scope

- persisted vector indices,
- ANN as a required first step,
- LLM tree search or generated summaries,
- OCR/scanned documents.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing Candle embedder | internal Rust module | Local pure-Rust embedding generation for queries and segments | current `dense` module |
| Existing extractors | internal Rust modules | Provide normalized text plus source-aware hints for segmentation | current `extract` module |
| `walkdir` | Rust crate | Deterministic corpus traversal | current dependency |
| Benchmark/eval harness | internal Rust modules | Quality and latency verification for the new retrieval path | current `bench` and `eval` modules |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Vector retrieval unit | Structure-aware segment, not whole document | Long documents need local semantic evidence |
| First vector search algorithm | Exact in-memory similarity scan | Keeps the first implementation simple and indexless |
| Segment-to-document scoring | Sum of retrieved segment scores divided by `sqrt(n + 1)` | Inspired by PageIndex; rewards multiple relevant sections with diminishing returns |
| Fusion method | Reciprocal Rank Fusion | More robust than direct score blending across independent retrievers |
| Final retrieval unit | Documents with best-section snippets | Matches sift's CLI contract while preserving semantic evidence |
| Runtime choice | Keep Candle as the default path | Preserves the repo's preferred pure-Rust story; `fastembed-rs` remains deferred |

## Architecture

The search stack becomes:

1. corpus loading extracts normalized document text;
2. a segment builder derives structure-aware segments per document;
3. BM25 still indexes document-level text for lexical retrieval;
4. a vector retriever embeds query + segments and scores all active segments;
5. a semantic aggregator converts segment hits into document-level semantic
   ranks and best-snippet references;
6. an RRF fuser combines BM25 and semantic document rankings;
7. the renderer returns documents with the best segment snippet.

## Components

- `Segment`
  Purpose: represent a searchable semantic unit beneath a document.
  Fields: `segment_id`, `doc_id`, `path`, `label`, `ordinal`, `text`,
  `source_kind`, and optional structural metadata such as page/slide/sheet.

- `SegmentBuilder`
  Purpose: derive structure-aware segments from `ExtractedDocument`.
  Behavior:
  - text: heading/paragraph or paragraph-window segmentation,
  - HTML: heading/section segmentation,
  - PDF: page-backed segmentation first,
  - PPTX: slide-backed segmentation,
  - XLSX: sheet/table-backed segmentation,
  - DOCX: heading/paragraph segmentation where recoverable.

- `VectorRetriever`
  Purpose: embed query and all active segments, then compute similarity scores.
  Behavior: exact scan over the in-memory segment vectors for the active search
  request.

- `SemanticAggregator`
  Purpose: aggregate segment hits into document-level scores and identify the
  best snippet source.
  Behavior: group hits by `doc_id`, compute a diminishing-returns document
  score, preserve the highest-signal segment for snippet rendering.

- `RankFuser`
  Purpose: fuse BM25 and semantic document rankings.
  Behavior: implement RRF over the lexical and semantic ranked lists.

- benchmark/eval extensions
  Purpose: measure quality/latency and record segment/vector metadata.
  Behavior: preserve exact command, hardware, git SHA, and configuration in the
  output report.

## Interfaces

Planned internal interface shape:

- `build_segments(document: &Document) -> Vec<Segment>`
- `score_segments(query: &str, segments: &[Segment], model: &DenseModelSpec) -> Vec<SegmentHit>`
- `aggregate_segment_hits(hits: &[SegmentHit]) -> Vec<SemanticDocumentHit>`
- `fuse_rankings(bm25: &[RankedDocument], semantic: &[SemanticDocumentHit]) -> Vec<RankedDocument>`

The CLI surface should continue to expose `search --engine hybrid`; the meaning
of the engine changes from reranking to dual retrieval.

## Data Flow

`WalkDir` discovers files -> extractors normalize file content -> document text
is indexed for BM25 -> segment builder derives structure-aware sections -> query
embeds against all active segments -> top segment hits aggregate into document
semantic scores -> BM25 document ranks and semantic document ranks are fused via
RRF -> best segment snippet is rendered with the final document result.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Segment builder produces zero segments for a document | segment list is empty after building | fallback to one coarse document-backed segment where possible | continue search with reduced structural fidelity |
| Vector scoring fails for the active corpus | embedder/model load or similarity computation returns error | fail the command with explicit context | user can retry after fixing model/runtime issue |
| A semantic hit references a missing document or segment | integrity checks during aggregation/rendering | fail fast in tests, surface explicit runtime error if encountered | fix the segment/document lineage bug before rollout |
| Exact vector search exceeds latency target | benchmark evidence shows p50/p90/max shortfall | preserve explicit shortfall in reports | re-plan ANN or persistence as a follow-on slice |
| Structure-aware segmentation is too lossy for a format | fixture or eval behavior shows poor snippets/quality | keep the document searchable with a coarse fallback segmenter | add format-specific segmentation improvements in later stories |
