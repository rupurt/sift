---
id: VDVSF1bAN
title: Adopt The Facade In The Executable
type: feat
status: done
created_at: 2026-03-10T15:30:27
updated_at: 2026-03-10T15:59:07
scope: VDVQurZER/VDVRkNjgH
index: 3
started_at: 2026-03-10T15:53:09
completed_at: 2026-03-10T15:59:07
---

# Adopt The Facade In The Executable

## Summary

Rewire `src/main.rs` to consume the curated library facade while preserving the
current executable command surface and help contract.

## Acceptance Criteria

- [x] [SRS-03/AC-01] The executable builds and routes search execution through the curated library boundary instead of depending on newly unsupported internals. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && cargo check --bin sift && rg -n "Sift::builder" src/main.rs && ! rg -n "SearchRequest|run_search|LocalFileCorpusRepository|StrategyPresetRegistry|DenseReranker|Embedder|RetrieverPolicy|FusionPolicy|RerankingPolicy" src/main.rs', SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-03/AC-02] User-facing command names, argument shapes, and baseline behavior remain intact across the cutover. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && rg -n "Search\\(SearchCommand\\)|override_usage|after_help|retrievers: Option<Vec<SearchRetriever>>|fusion: Option<SearchFusion>|reranking: Option<SearchReranking>|value = \\\"bm25\\\"|value = \\\"phrase\\\"|value = \\\"vector\\\"|value = \\\"rrf\\\"|value = \\\"none\\\"|value = \\\"position-aware\\\"|value = \\\"llm\\\"" src/main.rs', SRS-03:start:end, proof: ac-2.log-->
