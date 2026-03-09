# Vector Embedding Caching - Software Requirements Specification

> Update Segment model and corpus loading to compute and store dense embeddings in the global blob store.

## Scope

### In Scope

- [SCOPE-01] Add embedding storage to the `Segment` domain model.
- [SCOPE-02] Compute embeddings during the corpus loading "Slow Path" (extraction miss).
- [SCOPE-03] Bypass embedding computation if pre-computed embeddings exist in the cached `Document`.
- [SCOPE-04] Update documentation.

### Out of Scope

- [SCOPE-05] Vector quantization or compression.
- [SCOPE-06] External vector database integration.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-06 | `Segment` MUST store an `Option<Vec<f32>>` for dense embeddings. | SCOPE-01 | FR-01 | Unit test for serialization. |
| SRS-07 | `load_search_corpus` MUST compute and populate embeddings for all segments before saving a document to the blob store. | SCOPE-02 | FR-01 | Integration test or trace verification. |
| SRS-08 | `SegmentVectorRetriever` MUST utilize pre-computed embeddings if present in the corpus. | SCOPE-03 | FR-01 | Timing verification at `-v`. |
<!-- END FUNCTIONAL_REQUIREMENTS -->
