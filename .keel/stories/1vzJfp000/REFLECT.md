---
created_at: 2026-03-08T14:23:15
---

# Reflection - Implement Raw File BM25 Search CLI

## Knowledge

### 1vzLal000: Separate benchmark IDs from recursive search IDs
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When the same BM25 core is reused for benchmark corpora and raw recursive filesystem search |
| **Insight** | Recursive search needs stable full-path identities to avoid basename collisions, while benchmark corpora still need stem-based IDs to match qrels manifests |
| **Suggested Action** | Keep a shared in-memory document/index layer, but let each loader define its own canonical document ID policy |
| **Applies To** | `src/search.rs`, future hybrid reranking, benchmark corpus loaders |
| **Linked Knowledge IDs** | 1vzLO0001 |
| **Observed At** | 2026-03-08T21:23:30Z |
| **Score** | 0.88 |
| **Confidence** | 0.95 |
| **Applied** | yes |

## Observations

The story stayed tractable once the BM25/tokenization logic moved out of `bench`
and into a shared search module. That kept the benchmark behavior intact while
letting the real CLI add recursive traversal, snippets, and JSON output without
duplicating ranking code.

The main edge case was proof design for the no-sidecar requirement. Unit tests
covered deterministic skipping of invalid UTF-8, but the CLI proof needed an
explicit read-only signal, so I recorded a before/after corpus digest alongside
the live search command.
