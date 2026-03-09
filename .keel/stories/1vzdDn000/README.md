---
id: 1vzdDn000
title: Add Query Expansion And Phrase Retrieval
type: feat
status: done
created_at: 2026-03-09T09:12:43
updated_at: 2026-03-09T09:41:22
scope: 1vzXLN000/1vzdCx000
index: 6
started_at: 2026-03-09T09:40:10
submitted_at: 2026-03-09T09:41:21
completed_at: 2026-03-09T09:41:22
---

# Add Query Expansion And Phrase Retrieval

## Summary

Introduce a configurable query-expansion phase and first-class lexical retriever
adapters so BM25 remains the baseline while phrase/proximity retrieval becomes a
parallel option that can complement vector search.

## Acceptance Criteria

- [x] [SRS-03/AC-01] Named strategies can enable or disable query expansion and produce zero or more query variants through the shared search-plan model. <!-- verify: cargo test search::application::tests::search_service_orchestrates_multiple_variants_and_retrievers, SRS-03:start:end, proof: ac-1.log -->
- [x] [SRS-04/AC-02] Multiple retrievers can run independently over the same prepared corpus state and return separate candidate lists for later fusion. <!-- verify: cargo test search::application::tests::search_service_fuses_results_from_multiple_retrievers, SRS-04:start:end, proof: ac-2.log -->
- [x] [SRS-05/AC-03] The lexical retriever set includes `bm25` as the stable baseline and at least one phrase/proximity-aware retriever. <!-- verify: manual, SRS-05:start:end, proof: ac-3.log -->
- [x] [SRS-06/AC-04] Structure-aware segment vector retrieval is adapted into the shared retriever layer as an independent retriever rather than a reranking stage. <!-- verify: manual, SRS-06:start:end, proof: ac-4.log -->
