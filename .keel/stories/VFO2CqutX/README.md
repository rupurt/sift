---
# system-managed
id: VFO2CqutX
status: done
created_at: 2026-03-30T14:00:53
updated_at: 2026-03-30T14:20:29
# authored
title: Emit Indexing And Embedding Progress Events
type: feat
operator-signal:
scope: VFO1icY5Z/VFO1uSaNE
index: 3
started_at: 2026-03-30T14:16:34
completed_at: 2026-03-30T14:20:29
---

# Emit Indexing And Embedding Progress Events

## Summary

Wire progress callback into corpus loading (load_search_corpus) and embedding (SearchService::execute) phases. Emit Indexing events with files_processed/files_total during walkdir and Embedding events with chunks_processed/chunks_total during vector retrieval.

## Acceptance Criteria

- [x] [SRS-05/AC-01] Corpus loading emits Indexing progress with monotonically increasing files_processed up to files_total <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && grep -n "SearchProgress::Indexing" src/search/corpus.rs', SRS-05:start:end, proof: ac-1.log -->
- [x] [SRS-06/AC-02] Embedding phase emits Embedding progress with chunks_processed/chunks_total <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && grep -n "SearchProgress::Embedding" src/facade.rs', SRS-06:start:end, proof: ac-2.log -->
- [x] [SRS-05/AC-03] Indexing progress files_total matches actual file count in corpus <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && grep -n "files_total" src/search/corpus.rs', SRS-05:start:end, proof: ac-3.log -->
- [x] [SRS-06/AC-04] Embedding events are emitted separately from Indexing events <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && grep -n "SearchProgress::Embedding\|SearchProgress::Indexing" src/facade.rs src/search/corpus.rs', SRS-06:start:end, proof: ac-4.log -->
