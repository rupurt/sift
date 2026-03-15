# VOYAGE REPORT: Vector Embedding Caching

## Voyage Metadata
- **ID:** 1vzgQK000
- **Epic:** 1vzfew000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Add Embedding To Segment Model
- **ID:** 1vzgQX000
- **Status:** done

#### Acceptance Criteria
- [x] [SRS-06/AC-01] Add `embedding: Option<Vec<f32>>` to `Segment` in `src/segment.rs`. <!-- verify: manual, SRS-06:start:end, proof: ac-1.log -->
- [x] [SRS-06/AC-02] Update `build_segments` to initialize embedding as `None`. <!-- verify: manual, SRS-06:start:end, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzgQX000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzgQX000/EVIDENCE/ac-2.log)

### Compute And Cache Embeddings On Load
- **ID:** 1vzgQd000
- **Status:** done

#### Acceptance Criteria
- [x] [SRS-07/AC-01] Modify `load_search_corpus` to accept an optional `DenseReranker`. <!-- verify: manual, SRS-07:start:end, proof: ac-1.log -->
- [x] [SRS-07/AC-02] In the "Slow Path" (extraction miss), use the reranker to populate segment embeddings before calling `save_blob`. <!-- verify: manual, SRS-07:start:end, proof: ac-2.log -->
- [x] [SRS-08/AC-01] Update `SegmentVectorRetriever` to skip inference for segments that already have embeddings. <!-- verify: manual, SRS-08:start:end, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzgQd000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzgQd000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzgQd000/EVIDENCE/ac-3.log)

### Document Vector Caching
- **ID:** 1vzgQk000
- **Status:** done

#### Acceptance Criteria
- [x] [SRS-04/AC-01] Update `ARCHITECTURE.md` to explain that blobs contain fully processed assets (text + term stats + dense embeddings). <!-- verify: manual, SRS-04:start:end, proof: ac-1.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzgQk000/EVIDENCE/ac-1.log)


