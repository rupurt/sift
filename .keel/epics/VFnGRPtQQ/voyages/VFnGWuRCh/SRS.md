# Implement Sectorized Direct Search Reuse - SRS

## Summary

Epic: VFnGRPtQQ
Goal: Mount clean sectors on direct-search startup, validate them cheaply from existing cache substrate state, and rebuild only dirty sectors instead of rescanning the full corpus.

## Scope

### In Scope

- [SCOPE-01] Persist sector cache records and deterministic partitioning helpers under the existing search cache root.
- [SCOPE-02] Load clean sector artifacts and lexical shards during direct-search startup before falling back to rebuild work.
- [SCOPE-03] Rebuild dirty sectors in isolation and persist refreshed sector-local shard outputs.
- [SCOPE-04] Prove warm direct-search restart reuse over unchanged and partially changed corpora.

### Out of Scope

- [SCOPE-05] Breadcrumb journaling and interrupted-run resume.
- [SCOPE-06] Frontier/converging/sealed coverage semantics.
- [SCOPE-07] Controller or autonomous runtime adoption.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Define persisted `SectorMap` and sector-shard records with deterministic sector ids, membership summaries, validity proof material, and shard references. | SCOPE-01 | FR-01 | test |
| SRS-02 | Direct-search startup loads clean sectors from persisted state before rebuilding dirty sectors. | SCOPE-02 | FR-01 | test |
| SRS-03 | Dirty sectors rebuild in isolation and persist refreshed sector-local lexical shard outputs without invalidating unrelated clean sectors. | SCOPE-03 | FR-01 | test |
| SRS-04 | Warm direct-search restart over an unchanged corpus can begin from mounted clean sectors without a whole-corpus reparse before first useful results. | SCOPE-04 | FR-01 | command |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The implementation extends the existing manifest/blob/BM25 cache substrate instead of introducing a parallel file-state authority. | SCOPE-01, SCOPE-03 | NFR-02 | manual |
| SRS-NFR-02 | The voyage delivers direct-search value first without requiring controller or autonomous runtime changes. | SCOPE-02, SCOPE-04 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
