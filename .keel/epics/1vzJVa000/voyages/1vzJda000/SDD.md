# Build Hybrid Text Retrieval MVP - Software Design Description

> Deliver raw-document ASCII and UTF-8 hybrid search, an evaluation corpus pipeline, and benchmark evidence for the indexless MVP.

**SRS:** [SRS.md](SRS.md)

## Overview

The MVP keeps `sift` fully indexless at rest. Each search command walks the
filesystem, decodes supported text files, builds transient lexical structures
for the current process, and ranks the corpus in two stages:

1. BM25 scores the full candidate set.
2. A local pure-Rust encoder reranks only the top lexical candidates.

Evaluation and benchmark commands use the same retrieval pipeline against a
materialized SciFact corpus so the product can compare BM25-only and hybrid
behavior with exact recorded evidence.

## Context & Boundaries

<!-- What's in scope, what's out of scope, external actors/systems we interact with -->

```
┌───────────────────────────────────────────────────────┐
│                     This Voyage                       │
│                                                       │
│  filesystem/eval corpus -> decode -> tokenize        │
│            |                    |                     │
│            v                    v                     │
│      transient docs ------> BM25 full scan           │
│                                    |                 │
│                                    v                 │
│                         shortlist for dense rerank   │
│                                    |                 │
│                                    v                 │
│                   Candle-based local encoder         │
│                                    |                 │
│                                    v                 │
│                       hybrid fusion + snippets       │
└───────────────────────────────────────────────────────┘
           ↑                                 ↑
   Hugging Face corpus                 benchmark commands
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `bm25` | Rust crate | Sparse ranking over transient document representations | current crate version in `Cargo.toml` |
| Candle ecosystem | Rust crates | Pure-Rust local inference for dense reranking | latest compatible versions selected during implementation |
| Hugging Face model/tokenizer assets | model data | MiniLM-family sentence embedding weights and tokenizer | `sentence-transformers/all-MiniLM-L6-v2` or compatible |
| BEIR SciFact | evaluation corpus | Quality and latency benchmark corpus | Hugging Face dataset mirror |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| At-rest architecture | No persisted search sidecar index | Required by the operating contract and simplifies invalidation semantics. |
| Retrieval shape | BM25 over full corpus plus dense rerank on shortlist | Gives hybrid ranking while keeping dense inference bounded. |
| Dense runtime | Candle-first pure-Rust path | Fastest route to a defensible CPU-only local model implementation. |
| Evaluation corpus | Materialized BEIR SciFact text corpus | Standard qrels-based evaluation with manageable size. |
| Fusion | Simple combined reranking, likely reciprocal-rank or weighted score fusion | Easy to reason about and verify without introducing heavy dependencies. |

## Architecture

- `cli`: parses commands for search, corpus materialization, and benchmarking.
- `corpus`: downloads or reads eval assets, materializes text files, and loads
  queries/qrels.
- `fs`: walks directories, filters supported files, and reads UTF-8/ASCII
  content.
- `text`: normalizes documents, chunks content if needed, and extracts snippets.
- `sparse`: tokenizes documents and computes BM25 scores.
- `dense`: loads the local encoder and computes query/document embeddings.
- `fusion`: combines sparse and dense ranks into final result ordering.
- `bench`: runs repeatable quality and latency evaluations and records evidence.

## Components

- `SearchCommand`
  Purpose: end-user entrypoint for interactive retrieval.
  Interface: `sift search <query> [path] [--json] [--bm25-only]`.
  Behavior: enumerates files, builds transient candidates, scores, fuses, and
  prints results.
- `EvalCorpusCommand`
  Purpose: fetch and materialize SciFact into a local filesystem-shaped corpus.
  Interface: `sift eval download ...` and `sift eval materialize ...`.
  Behavior: downloads corpus/query/qrels assets, writes UTF-8 text files and
  manifests, and keeps IDs stable for evaluation.
- `SparseRetriever`
  Purpose: compute lexical relevance across all documents.
  Interface: in-memory scoring API consumed by search and benchmark commands.
  Behavior: tokenizes the corpus and query for each process run without writing
  a persistent index.
- `DenseReranker`
  Purpose: compute embeddings for the query and shortlist candidates.
  Interface: rerank `Vec<Candidate>` with dense scores.
  Behavior: loads a local model once per process and encodes only bounded
  candidates to control latency.
- `BenchmarkRunner`
  Purpose: compare engines and capture reproducible evidence.
  Interface: latency and quality subcommands.
  Behavior: runs queries, computes metrics, and emits structured results with
  machine metadata.

## Interfaces

- `sift search`
- `sift search --json`
- `sift eval download`
- `sift eval materialize`
- `sift bench latency`
- `sift bench quality`

## Data Flow

1. The CLI receives either a local search query or a benchmark/eval command.
2. For search, the filesystem walker discovers supported files under the target
   path and reads them into transient document structures.
3. The sparse retriever tokenizes documents and computes BM25 scores across the
   full corpus.
4. The top lexical candidates are forwarded to the dense reranker.
5. The dense reranker encodes the query once and the shortlisted candidates via
   the local pure-Rust encoder.
6. The fusion layer combines sparse and dense signals into final ranking.
7. Output rendering returns human-readable or JSON results with snippets.
8. For benchmark commands, the same path repeats across many queries and writes
   structured metrics plus environment metadata.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Unsupported or non-UTF-8 file | Decode failure or filter rejection | Skip file with counted diagnostic, do not crash search. | Report skipped-file counts in benchmark metadata. |
| Corpus download fails | HTTP or file I/O error | Exit with actionable error message. | Retry command or use pre-downloaded assets. |
| Model asset load fails | Missing tokenizer/weights or incompatible format | Exit before search starts. | Re-download assets or switch configured model. |
| Hybrid path exceeds latency target | Benchmark evidence shows p50/p90 above target | Record exact result rather than masking it. | Tighten shortlist, adjust model, or re-plan. |
