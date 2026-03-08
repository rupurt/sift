---
id: 1vzJfp000
title: Implement Raw File BM25 Search CLI
type: feat
scope: 1vzJVa000/1vzJda000
status: done
created_at: 2026-03-08T12:20:21
updated_at: 2026-03-08T14:25:15
started_at: 2026-03-08T14:14:02
completed_at: 2026-03-08T14:25:15
---

# Implement Raw File BM25 Search CLI

## Summary

Implement the first real search path for `sift`: recursive raw-file traversal,
ASCII/UTF-8 decoding, transient BM25 ranking, snippets, and both terminal and
JSON output without any persisted search index.

## Acceptance Criteria

- [x] [SRS-04/AC-01] `sift search <query> <path>` recursively scans supported ASCII/UTF-8 files and returns ranked BM25 results without requiring a prebuilt or persisted index. <!-- verify: sh -lc 'cargo test search::bm25 && cargo run -- search "retrieval architecture" .cache/eval/scifact-files --engine bm25', SRS-04:start:end, proof: ac-1.log -->
- [x] [SRS-05/AC-01] `sift search --json` emits machine-readable result entries containing path, rank, score, and snippet fields for agent consumption. <!-- verify: sh -lc 'cargo test cli::json_output && cargo run -- search --json "retrieval architecture" .cache/eval/scifact-files --engine bm25', SRS-05:start:end, proof: ac-2.log -->
- [x] [SRS-06/AC-01] Search execution does not create a persisted sidecar index or require a background service, and unsupported files are skipped deterministically rather than crashing the command. <!-- verify: sh -lc 'before=$(find .cache/eval/scifact-files -type f -print | LC_ALL=C sort | xargs sha256sum | sha256sum | cut -d" " -f1); cargo test fs::filtering && cargo run -- search "retrieval architecture" .cache/eval/scifact-files --engine bm25 >/tmp/sift-search-ac3.txt; after=$(find .cache/eval/scifact-files -type f -print | LC_ALL=C sort | xargs sha256sum | sha256sum | cut -d" " -f1); printf "corpus_digest_before=%s\ncorpus_digest_after=%s\n\n" "$before" "$after"; cat /tmp/sift-search-ac3.txt', SRS-06:start:end, proof: ac-3.log -->
