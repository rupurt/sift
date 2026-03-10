---
id: VDPyDiXga
title: Implement Session-Level Query Embedding Cache
type: feat
status: in-progress
created_at: 2026-03-09T16:59:50
updated_at: 2026-03-09T17:01:22
scope: VDPy8MNer/VDPyAtjbT
index: 1
started_at: 2026-03-09T17:01:22
---

# Implement Session-Level Query Embedding Cache

## Summary

Reduce redundant neural network inference by caching query embeddings at the session level during search.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `DenseReranker` implements query caching via a session-level store <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [ ] [SRS-01/AC-02] Repeated searches for the same query within a search session avoid re-embedding <!-- verify: manual, SRS-01:start:end, proof: ac-2.log-->
- [ ] [SRS-04/AC-03] `sift search -vv` shows cache hits for repeated queries in its trace output <!-- verify: command, SRS-04:start:end, proof: ac-3.log-->
- [ ] [SRS-01/AC-04] Search results are identical with and without the cache enabled <!-- verify: manual, SRS-01:start:end, proof: ac-4.log-->
