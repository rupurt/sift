---
# system-managed
id: VG7XsurqV
status: done
created_at: 2026-04-07T08:48:58
updated_at: 2026-04-07T08:59:43
# authored
title: Implement Structural Fuzzy Retrieval Substrate
type: feat
operator-signal:
scope: VG7WSIxMB/VG7WmMIjI
index: 1
started_at: 2026-04-07T08:49:38
submitted_at: 2026-04-07T08:59:38
completed_at: 2026-04-07T08:59:43
---

# Implement Structural Fuzzy Retrieval Substrate

## Summary

Implement the structural fuzzy retrieval slice for the direct-search substrate:
add path-aware fuzzy retrieval, typo-tolerant fuzzy line/segment retrieval, and
real structural reranking; then document the shipped behavior and the downstream
`paddles` adoption seam in the foundational documents.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `RetrieverPolicy`, runtime registration, and built-in structural presets expose a path fuzzy retriever through the existing direct-search plan surface. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test default_registry_includes_structural_fuzzy_strategies && cargo test path_fuzzy_retriever_prefers_filename_like_queries', SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-02] The runtime exposes a fuzzy line/segment retriever that returns snippet-bearing evidence for typo-tolerant structural matches. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test segment_fuzzy_retriever_returns_snippet_evidence', SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-03] `PositionAwareReranker` applies deterministic bonuses for path, heading, and definition-like evidence instead of acting as a passthrough. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test position_aware_reranker_boosts_structural_matches', SRS-03:start:end, proof: ac-3.log -->
- [x] [SRS-04/AC-04] Rich built-in presets and public `SearchPlan` helpers expose the structural fuzzy retrieval stack without changing the direct-search boundary. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test search_plan_default_page_index_hybrid_includes_structural_fuzzy_retrievers && cargo test default_registry_includes_structural_fuzzy_strategies', SRS-04:start:end, proof: ac-4.log -->
- [x] [SRS-05/AC-05] Foundational docs describe the new retrievers, preset composition, reranker behavior, and downstream `paddles` adoption path. <!-- verify: manual, SRS-05:start:end, proof: ac-5.log -->
