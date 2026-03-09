---
id: 1vzSw1000
title: Add Full-Corpus Segment Vector Retrieval
type: feat
scope: 1vzSne000/1vzSy6000
status: backlog
created_at: 2026-03-08T22:13:41
updated_at: 2026-03-08T22:15:14
---

# Add Full-Corpus Segment Vector Retrieval

## Summary

Add a corpus-wide vector retriever that embeds and scores structure-aware
segments across the active corpus, then aggregates segment hits into
document-level semantic scores without writing a persisted vector index.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] Vector retrieval scores the full active segment corpus instead of scoring only BM25-shortlisted documents. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test vector_retrieval::full_corpus && cargo run -- search tests/fixtures/rich-docs "semantic retrieval" --engine hybrid', SRS-03:start:end, proof: ac-1.log -->
- [ ] [SRS-04/AC-01] Segment-level vector hits aggregate into document-level semantic scores through the planned diminishing-returns rule. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test vector_retrieval::aggregation', SRS-04:start:end, proof: ac-2.log -->
