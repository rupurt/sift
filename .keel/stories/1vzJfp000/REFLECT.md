---
created_at: 2026-03-08T14:23:15
---

# Reflection - Implement Raw File BM25 Search CLI

## Knowledge

- [1vzLal000](../../knowledge/1vzLal000.md) Separate benchmark IDs from recursive search IDs

## Observations

The story stayed tractable once the BM25/tokenization logic moved out of `bench`
and into a shared search module. That kept the benchmark behavior intact while
letting the real CLI add recursive traversal, snippets, and JSON output without
duplicating ranking code.

The main edge case was proof design for the no-sidecar requirement. Unit tests
covered deterministic skipping of invalid UTF-8, but the CLI proof needed an
explicit read-only signal, so I recorded a before/after corpus digest alongside
the live search command.
