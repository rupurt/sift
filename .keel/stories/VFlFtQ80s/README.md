---
# system-managed
id: VFlFtQ80s
status: done
created_at: 2026-04-03T13:19:31
updated_at: 2026-04-03T13:28:37
# authored
title: Add Incremental Indexing Progress And Reuse
type: feat
operator-signal:
scope: VFlF7aHQw/VFlFAS7rn
index: 1
started_at: 2026-04-03T13:20:14
completed_at: 2026-04-03T13:28:37
---

# Add Incremental Indexing Progress And Reuse

## Summary

Enable cache-backed incremental indexing in the shipped search paths and make the indexing phase legible to both CLI users and library embedders. This story covers direct-search progress plumbing, shared BM25 reuse across direct and autonomous flows, richer telemetry snapshots, and human-readable stderr progress for text-mode CLI runs.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `Sift::search_with_progress` emits direct-search progress and existing non-progress direct callers continue to work unchanged. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && cargo test facade::tests -- --nocapture', SRS-01:start:end, proof: ac-direct.log -->
- [x] [SRS-02/AC-02] Public telemetry snapshots expose the indexing metrics needed to explain cache reuse versus fresh work. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && cargo test facade::tests -- --nocapture', SRS-02:start:end, proof: ac-direct.log -->
- [x] [SRS-03/AC-03] Artifact loading records fresh extraction/build counts, skips, and BM25 cache/build counts. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && cargo test facade::tests -- --nocapture', SRS-03:start:end, proof: ac-direct.log -->
- [x] [SRS-04/AC-04] Direct search reuses the persisted BM25 index on an unchanged corpus when cache_dir is enabled. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && cargo test facade::tests -- --nocapture', SRS-04:start:end, proof: ac-direct.log -->
- [x] [SRS-05/AC-05] Autonomous/controller search reuses the persisted BM25 index on an unchanged corpus when cache_dir is enabled. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && cargo test facade::tests -- --nocapture', SRS-05:start:end, proof: ac-direct.log -->
- [x] [SRS-06/AC-06] The bundled CLI enables a default search cache root for normal `sift search` runs. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && cargo test search_cli_tests -- --nocapture && rg -n "with_cache_dir\\(cache_dir\\(\\\"search\\\"\\)\\?\\)" src/main.rs', SRS-06:start:end, proof: ac-cli.log -->
- [x] [SRS-07/AC-07] Text-mode CLI search writes live progress to stderr without changing stdout result rendering. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && cargo test search_cli_tests -- --nocapture && rg -n "search_autonomous_with_progress|search_with_progress|ProgressRenderer" src/main.rs', SRS-07:start:end, proof: ac-cli.log -->
- [x] [SRS-08/AC-08] Indexing progress rendering includes file counts plus cache/build metrics that explain current preparation work. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && cargo test search_cli_tests -- --nocapture && rg -n "blobs|fresh|bm25 cache|ProgressRenderer::format_line" src/main.rs', SRS-08:start:end, proof: ac-cli.log -->
