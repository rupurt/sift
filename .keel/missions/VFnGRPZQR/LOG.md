# Implement Sector-Aware Frontier Search Cache Reuse - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-04-03T21:45:00-07:00

- Created execution mission `VFnGRPZQR` and epic `VFnGRPtQQ` to implement sector-aware frontier search cache reuse.
- Decomposed the work into four voyages covering direct-search sector reuse, resumable breadcrumb journaling, truthful frontier coverage semantics, and cross-surface runtime adoption.
- Authored story-level acceptance criteria so the mission can be attached, planned, and activated as executable board demand.

## 2026-04-03T21:54:02-07:00

- Completed and accepted story `VFnGb5zp9` for the direct-search voyage.
- Added `src/cache/sector.rs` with persisted `SectorMap` records, deterministic stable-hash partitioning, proof material, and lexical shard references keyed off the existing corpus cache identity.
- Refactored locked bincode persistence into shared cache-store helpers and reused the shared corpus cache key in the existing BM25/manifests layout to keep the new sector substrate on the same cache authority.
- Verified the slice with `cargo test sector` and a full `cargo test` run, leaving `VFnGb64p8` as the next direct-search execution story.

## 2026-04-03T22:12:11-07:00

- Completed implementation story `VFnGb64p8` for sector-aware direct-search startup.
- Reworked `src/search/corpus.rs` to load clean sectors from persisted blobs, rebuild only dirty/untracked sectors, and persist a refreshed `SectorMap` plus sector-local BM25 shards from actual loaded artifacts.
- Updated `src/search/application.rs` to combine persisted sector BM25 shards on whole-corpus cache misses, and extended telemetry/progress surfaces with sector cache and sector shard counters.
- Added direct regression coverage for warm clean-sector restart reuse and one-sector dirty rebuild isolation, then verified the full workspace with `cargo test`.

## 2026-04-03T22:21:07-07:00

- Completed breadcrumb persistence story `VFnGb6Nq5`.
- Added `src/cache/breadcrumb.rs` for persisted rebuild journals keyed under the existing cache root and wired dirty-sector rebuild checkpoints through `src/search/corpus.rs`.
- Added focused breadcrumb persistence tests and re-ran the full workspace test suite with `cargo test`.

## 2026-04-03T22:29:26-07:00

- Completed story `VFnGb6Xq1` for startup breadcrumb resume and recovery.
- Updated `src/search/corpus.rs` to persist manifest checkpoints during dirty-sector rebuilds, validate saved breadcrumb journals on startup, resume completed and partially processed sectors from cached blobs, and discard stale or corrupt journals safely.
- Extended `src/system.rs`, `src/search/domain.rs`, and `src/main.rs` with breadcrumb resume and recovery telemetry so both library embedders and the interactive CLI can observe indexing restart behavior.
- Added restart regression tests for resume, stale-journal discard, and corrupt-journal discard, then re-ran the full workspace with `cargo test`.
