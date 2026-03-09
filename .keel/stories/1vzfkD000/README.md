---
id: 1vzfkD000
title: Wire Cache Into Prepared Corpus
type: feat
status: done
scope: 1vzfew000/1vzfjD000
created_at: 2026-03-09T11:21:45
updated_at: 2026-03-09T11:59:53
started_at: 2026-03-09T11:59:26
submitted_at: 2026-03-09T11:59:40
completed_at: 2026-03-09T11:59:53
---

# Wire Cache Into Prepared Corpus

## Context

The extracted blobs need to be wired into `src/search/corpus.rs` to bypass text extraction and dense vector computation on cache hit.

## Acceptance Criteria

- [x] [SRS-04/AC-02] Update `load_search_corpus` to instantiate the manifest for the search root and query it for each file. <!-- verify: manual, SRS-04:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-04] When the manifest misses but the blake3 blob exists, update the manifest and load from cache. <!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-05] Only fallback to `extract_file` when the blob store has a miss. Ensure the newly extracted document is saved back to the blob store and manifest is updated. <!-- verify: manual, SRS-02:start:end, proof: ac-3.log -->
