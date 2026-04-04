---
# system-managed
id: VFnGb5zp9
status: done
created_at: 2026-04-03T21:34:51
updated_at: 2026-04-03T21:53:42
# authored
title: Define Sector Cache Models And Partitioning
type: feat
operator-signal:
scope: VFnGRPtQQ/VFnGWuRCh
index: 1
started_at: 2026-04-03T21:48:24
submitted_at: 2026-04-03T21:53:35
completed_at: 2026-04-03T21:53:42
---

# Define Sector Cache Models And Partitioning

## Summary

Define the persisted sector cache models and deterministic partitioning helpers that direct-search startup will rely on for clean-sector reuse and isolated dirty-sector rebuilds.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Sector cache models define deterministic sector ids, membership summaries, validity proof material, and lexical shard references. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test sector', SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-02] Sector cache records extend the existing manifest/blob/BM25 cache substrate instead of introducing a parallel file-state authority. <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-2.log-->
