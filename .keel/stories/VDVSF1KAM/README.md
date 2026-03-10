---
id: VDVSF1KAM
title: Decouple Public API From CLI Concerns
type: feat
status: done
created_at: 2026-03-10T15:30:27
updated_at: 2026-03-10T15:52:51
scope: VDVQurZER/VDVRkNjgH
index: 2
started_at: 2026-03-10T15:48:50
completed_at: 2026-03-10T15:52:51
---

# Decouple Public API From CLI Concerns

## Summary

Narrow the supported library API so it no longer depends on CLI parsing,
terminal rendering, or other executable-only concerns in public types.

## Acceptance Criteria

- [x] [SRS-02/AC-01] Supported public library types used by embedders do not require `clap` derives or CLI-only enums in their contract. <!-- verify: cargo check --test library_facade_test, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-02] Terminal rendering helpers and similar CLI presentation concerns are no longer part of the canonical embedded API path. <!-- verify: sh -lc '! rg -n "render_search_response|RetrieverPolicy|FusionPolicy|RerankingPolicy" src/lib.rs', SRS-02:start:end, proof: ac-2.log-->
