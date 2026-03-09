---
id: 1vzfkD000
title: Wire Cache Into Prepared Corpus
type: feat
status: backlog
scope: 1vzfew000/1vzfjD000
created_at: 2026-03-09T11:21:45
updated_at: 2026-03-09T11:54:55
---

# Wire Cache Into Prepared Corpus

## Context

The extracted blobs need to be wired into `src/search/corpus.rs` to bypass text extraction and dense vector computation on cache hit.

## Acceptance Criteria

- [ ] [SRS-04/AC-02] Update `load_search_corpus` to instantiate the manifest for the search root and query it for each file.
- [ ] [SRS-02/AC-04] When the manifest misses but the blake3 blob exists, update the manifest and load from cache.
- [ ] [SRS-02/AC-05] Only fallback to `extract_file` when the blob store has a miss. Ensure the newly extracted document is saved back to the blob store and manifest is updated.
