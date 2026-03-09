# VOYAGE REPORT: Structure-Aware True Hybrid Retrieval

## Voyage Metadata
- **ID:** 1vzSy6000
- **Epic:** 1vzSne000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Add Structure-Aware Segment Model
- **ID:** 1vzSvm000
- **Status:** done

#### Summary
Introduce the structure-aware segment abstraction beneath the current document
loader and build source-aware segment plans for the supported document
families.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] The corpus pipeline emits stable document and segment identifiers and at least one segment for every supported searchable document. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test segment_identity && cargo test structure_aware_segments', SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-01] Structure-aware segments preserve section-local text that can be used later for semantic retrieval and best-section snippets across text and rich-document fixtures. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test segment_text_preservation && cargo run -- search --json tests/fixtures/rich-docs "service catalog" --engine bm25', SRS-02:start:end, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzSvm000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzSvm000/EVIDENCE/ac-2.log)

### Add Full-Corpus Segment Vector Retrieval
- **ID:** 1vzSw1000
- **Status:** done

#### Summary
Add a corpus-wide vector retriever that embeds and scores structure-aware
segments across the active corpus, then aggregates segment hits into
document-level semantic scores without writing a persisted vector index.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Vector retrieval scores the full active segment corpus instead of scoring only BM25-shortlisted documents. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test vector_retrieval::full_corpus && cargo run -- search tests/fixtures/rich-docs "semantic retrieval" --engine hybrid', SRS-03:start:end, proof: ac-1.log -->
- [x] [SRS-04/AC-01] Segment-level vector hits aggregate into document-level semantic scores through the planned diminishing-returns rule. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test vector_retrieval::aggregation', SRS-04:start:end, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzSw1000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzSw1000/EVIDENCE/ac-2.log)

### Fuse BM25 And Vector Retrieval In Hybrid Search
- **ID:** 1vzSwD000
- **Status:** done

#### Summary
Replace the current rerank-style hybrid path with BM25 document retrieval plus
vector document retrieval fused by Reciprocal Rank Fusion, and render
best-section snippets in the final document results.

#### Acceptance Criteria
- [x] [SRS-05/AC-01] `search --engine hybrid` fuses independent BM25 and vector document rankings with Reciprocal Rank Fusion. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test hybrid::rrf && cargo run -- search tests/fixtures/rich-docs "architecture decision" --engine hybrid', SRS-05:start:end, proof: ac-1.log -->
- [x] [SRS-06/AC-01] Hybrid search returns document-level results with snippets sourced from the best matching segment rather than from an arbitrary whole-document truncation. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test hybrid::best_segment_snippet && cargo run -- search --json tests/fixtures/rich-docs "quarterly roadmap" --engine hybrid', SRS-06:start:end, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzSwD000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzSwD000/EVIDENCE/ac-2.log)

### Benchmark True Hybrid Retrieval
- **ID:** 1vzSwK000
- **Status:** done

#### Summary
Extend the benchmark and evaluation harnesses so they measure the true-hybrid
architecture, record the new vector/segment configuration, and make any latency
tradeoffs explicit.

#### Acceptance Criteria
- [x] [SRS-07/AC-01] Benchmark and evaluation commands compare BM25-only retrieval with the true-hybrid path and report the resulting metric deltas. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test bench::quality && cargo run -- bench quality --engine hybrid --baseline bm25 --corpus tests/fixtures/rich-docs --queries tests/fixtures/rich-docs/test-queries.tsv --qrels tests/fixtures/rich-docs/qrels/test.tsv', SRS-07:start:end, proof: ac-1.log -->
- [x] [SRS-08/AC-01] Benchmark output records the segment configuration, embedding model settings, command line, git SHA, corpus shape, and hardware summary for the true-hybrid path. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test bench::metadata && cargo run -- bench latency --engine hybrid --corpus tests/fixtures/rich-docs --queries tests/fixtures/rich-docs/test-queries.tsv', SRS-08:start:end, proof: ac-2.log -->
- [x] [SRS-09/AC-01] The true-hybrid implementation does not create or require a persisted vector sidecar index or background service. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! find . -path "./target" -prune -o -path "./.git" -prune -o -name "*.idx" -o -name "*.faiss" -o -name "*.ann" -o -name "*.hnsw" -print | rg . && ./target/release/sift search tests/fixtures/rich-docs "semantic retrieval" --engine hybrid', SRS-09:start:end, proof: ac-3.log -->
- [x] [SRS-10/AC-01] The default vector retrieval runtime remains the local pure-Rust Candle path rather than introducing `fastembed-rs` or ONNX Runtime as the default dependency. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo tree | rg "candle" && ! cargo tree | rg " fastembed|\\bort\\b"', SRS-10:start:end, proof: ac-4.log -->
- [x] [SRS-11/AC-01] Latency reporting makes any shortfall against the 200 ms target explicit for the true-hybrid path. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test bench::latency && cargo run -- bench latency --engine hybrid --corpus tests/fixtures/rich-docs --queries tests/fixtures/rich-docs/test-queries.tsv', SRS-11:start:end, proof: ac-5.log -->

#### Verified Evidence
- [ac-4.log](../../../../stories/1vzSwK000/EVIDENCE/ac-4.log)
- [ac-1.log](../../../../stories/1vzSwK000/EVIDENCE/ac-1.log)
- [ac-5.log](../../../../stories/1vzSwK000/EVIDENCE/ac-5.log)
- [ac-3.log](../../../../stories/1vzSwK000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vzSwK000/EVIDENCE/ac-2.log)


