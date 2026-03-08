---
id: 1vzJfp000
title: Implement Raw File BM25 Search CLI
type: feat
status: icebox
created_at: 2026-03-08T12:20:21
updated_at: 2026-03-08T12:20:21
---

# Implement Raw File BM25 Search CLI

## Summary

Implement the first real search path for `sift`: recursive raw-file traversal,
ASCII/UTF-8 decoding, transient BM25 ranking, snippets, and both terminal and
JSON output without any persisted search index.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] `sift search <query> <path>` recursively scans supported ASCII/UTF-8 files and returns ranked BM25 results without requiring a prebuilt or persisted index. <!-- verify: cargo test search::bm25 + cargo run -- search "retrieval architecture" .cache/eval/scifact-files --engine bm25, SRS-02:start:end, proof: ac-1.log -->
- [ ] [SRS-05/AC-01] `sift search --json` emits machine-readable result entries containing path, rank, score, and snippet fields for agent consumption. <!-- verify: cargo test cli::json_output + cargo run -- search --json "retrieval architecture" .cache/eval/scifact-files --engine bm25, SRS-05:start:end, proof: ac-2.log -->
- [ ] [SRS-06/AC-01] Search execution does not create a persisted sidecar index or require a background service, and unsupported files are skipped deterministically rather than crashing the command. <!-- verify: cargo test fs::filtering + cargo run -- search "retrieval architecture" .cache/eval/scifact-files --engine bm25, SRS-06:start:end, proof: ac-3.log -->
