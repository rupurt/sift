---
id: 1vzdDu000
title: Add Fusion And Reranking Layers
type: feat
status: done
created_at: 2026-03-09T09:12:50
updated_at: 2026-03-09T09:42:55
scope: 1vzXLN000/1vzdCx000
index: 7
started_at: 2026-03-09T09:41:35
submitted_at: 2026-03-09T09:42:55
completed_at: 2026-03-09T09:42:55
---

# Add Fusion And Reranking Layers

## Summary

Separate result fusion from reranking so hybrid strategies can combine multiple
retrievers with RRF by default and leave reranking as a bounded optional stage
behind its own port.

## Acceptance Criteria

- [x] [SRS-07/AC-01] The shared fusion layer uses Reciprocal Rank Fusion by default and preserves contributor provenance for explanation and benchmarking. <!-- verify: cargo test search::adapters::tests::rrf_fuser_preserves_provenance, SRS-07:start:end, proof: ac-1.log -->
- [x] [SRS-08/AC-02] The reranking layer is optional, exposed behind a reranker port, and ships with `none` as the default implementation. <!-- verify: manual, SRS-08:start:end, proof: ac-2.log -->
- [x] [SRS-13/AC-03] The default fusion and reranking stack remains local-first and does not require external databases or resident services. <!-- verify: manual, SRS-13:start:end, proof: ac-3.log -->
