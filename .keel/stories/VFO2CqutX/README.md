---
# system-managed
id: VFO2CqutX
status: backlog
created_at: 2026-03-30T14:00:53
updated_at: 2026-03-30T14:03:42
# authored
title: Emit Indexing And Embedding Progress Events
type: feat
operator-signal:
scope: VFO1icY5Z/VFO1uSaNE
index: 3
---

# Emit Indexing And Embedding Progress Events

## Summary

Wire progress callback into corpus loading (load_search_corpus) and embedding (SearchService::execute) phases. Emit Indexing events with files_processed/files_total during walkdir and Embedding events with chunks_processed/chunks_total during vector retrieval.

## Acceptance Criteria

- [ ] [SRS-05/AC-01] Corpus loading emits Indexing progress with monotonically increasing files_processed up to files_total <!-- verify: test, SRS-05 -->
- [ ] [SRS-06/AC-02] Embedding phase emits Embedding progress with chunks_processed/chunks_total <!-- verify: test, SRS-06 -->
- [ ] [SRS-05/AC-03] Indexing progress files_total matches actual file count in corpus <!-- verify: test, SRS-05 -->
- [ ] [SRS-06/AC-04] Embedding events are emitted separately from Indexing events <!-- verify: test, SRS-06 -->
