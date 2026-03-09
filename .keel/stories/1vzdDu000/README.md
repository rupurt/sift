---
id: 1vzdDu000
title: Add Fusion And Reranking Layers
type: feat
status: backlog
created_at: 2026-03-09T09:12:50
updated_at: 2026-03-09T09:20:48
scope: 1vzXLN000/1vzdCx000
index: 7
---

# Add Fusion And Reranking Layers

## Summary

Separate result fusion from reranking so hybrid strategies can combine multiple
retrievers with RRF by default and leave reranking as a bounded optional stage
behind its own port.

## Acceptance Criteria

- [ ] [SRS-07/AC-01] The shared fusion layer uses Reciprocal Rank Fusion by
      default and preserves contributor provenance for explanation and
      benchmarking.
- [ ] [SRS-08/AC-02] The reranking layer is optional, exposed behind a
      reranker port, and ships with `none` as the default implementation.
- [ ] [SRS-13/AC-03] The default fusion and reranking stack remains local-first
      and does not require external databases or resident services.
