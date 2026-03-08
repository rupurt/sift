---
created_at: 2026-03-08T15:22:52
---

# Knowledge - 1vzJda000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Build SciFact Evaluation And Benchmark Harness (1vzJfc000)

### 1vzLO0001: Keep materialized eval corpora self-contained

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When a CLI benchmark command needs both document files and query/qrels metadata from a downloaded evaluation corpus |
| **Insight** | Copying the benchmark query TSV into the materialized corpus directory keeps benchmark commands path-local and avoids coupling later stories to the raw download tree layout |
| **Suggested Action** | For future corpora, materialize documents plus the benchmark-facing query/qrels files into one self-contained directory structure |
| **Applies To** | `src/eval.rs`, `src/bench.rs`, future eval corpus adapters |
| **Applied** | yes |



---

## Story: Add Candle Dense Reranking And Hybrid Fusion (1vzJfv000)

### 1vzMBJ000: Sequence Length Dominates Hybrid Reranker Latency

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Tuning Candle-based dense reranking for transient hybrid search over thousands of raw documents |
| **Insight** | On the SciFact workload, lowering the dense encoder `max_length` had a much larger latency effect than reducing the BM25 shortlist. `all-MiniLM-L6-v2` at `max_length 40` kept a material quality gain over BM25 while bringing full-run hybrid latency back under the p50/p90 target. |
| **Suggested Action** | Treat sequence length as the primary performance dial before weakening shortlist depth or replacing the model, and record both quality and full-query latency before changing defaults. |
| **Applies To** | `src/dense.rs`, `src/main.rs`, `src/bench.rs`, hybrid search benchmarks |
| **Applied** | yes |

### 1vzMBJ001: Keep Keel Verify Proofs Bounded

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Stories whose acceptance evidence depends on long-running full-corpus benchmark commands |
| **Insight** | `keel verify run` is better used for bounded smoke proofs and targeted tests; full release benchmarks are more reliable when recorded separately with `keel story record --cmd`, which preserves the exact command output in evidence logs without making the verifier brittle. |
| **Suggested Action** | Put the exact long benchmark commands in `story record` evidence, and keep README `verify:` annotations short enough to pass consistently under the verifier harness. |
| **Applies To** | `.keel/stories/*/README.md`, benchmark-heavy stories |
| **Applied** | yes |



---

## Story: Implement Raw File BM25 Search CLI (1vzJfp000)

### 1vzLal000: Separate benchmark IDs from recursive search IDs

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When the same BM25 core is reused for benchmark corpora and raw recursive filesystem search |
| **Insight** | Recursive search needs stable full-path identities to avoid basename collisions, while benchmark corpora still need stem-based IDs to match qrels manifests |
| **Suggested Action** | Keep a shared in-memory document/index layer, but let each loader define its own canonical document ID policy |
| **Applies To** | `src/search.rs`, future hybrid reranking, benchmark corpus loaders |
| **Applied** | yes |



---

## Synthesis

### fje4WcUfh: Keep materialized eval corpora self-contained

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When a CLI benchmark command needs both document files and query/qrels metadata from a downloaded evaluation corpus |
| **Insight** | Copying the benchmark query TSV into the materialized corpus directory keeps benchmark commands path-local and avoids coupling later stories to the raw download tree layout |
| **Suggested Action** | For future corpora, materialize documents plus the benchmark-facing query/qrels files into one self-contained directory structure |
| **Applies To** | `src/eval.rs`, `src/bench.rs`, future eval corpus adapters |
| **Linked Knowledge IDs** | 1vzLO0001 |
| **Score** | 0.82 |
| **Confidence** | 0.91 |
| **Applied** | yes |

### 3jqrHI2yB: Sequence Length Dominates Hybrid Reranker Latency

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Tuning Candle-based dense reranking for transient hybrid search over thousands of raw documents |
| **Insight** | On the SciFact workload, lowering the dense encoder `max_length` had a much larger latency effect than reducing the BM25 shortlist. `all-MiniLM-L6-v2` at `max_length 40` kept a material quality gain over BM25 while bringing full-run hybrid latency back under the p50/p90 target. |
| **Suggested Action** | Treat sequence length as the primary performance dial before weakening shortlist depth or replacing the model, and record both quality and full-query latency before changing defaults. |
| **Applies To** | `src/dense.rs`, `src/main.rs`, `src/bench.rs`, hybrid search benchmarks |
| **Linked Knowledge IDs** | 1vzMBJ000 |
| **Score** | 0.94 |
| **Confidence** | 0.93 |
| **Applied** | yes |

### erhVFPQbp: Keep Keel Verify Proofs Bounded

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Stories whose acceptance evidence depends on long-running full-corpus benchmark commands |
| **Insight** | `keel verify run` is better used for bounded smoke proofs and targeted tests; full release benchmarks are more reliable when recorded separately with `keel story record --cmd`, which preserves the exact command output in evidence logs without making the verifier brittle. |
| **Suggested Action** | Put the exact long benchmark commands in `story record` evidence, and keep README `verify:` annotations short enough to pass consistently under the verifier harness. |
| **Applies To** | `.keel/stories/*/README.md`, benchmark-heavy stories |
| **Linked Knowledge IDs** | 1vzMBJ001 |
| **Score** | 0.76 |
| **Confidence** | 0.89 |
| **Applied** | yes |

### fNFOk3adt: Separate benchmark IDs from recursive search IDs

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When the same BM25 core is reused for benchmark corpora and raw recursive filesystem search |
| **Insight** | Recursive search needs stable full-path identities to avoid basename collisions, while benchmark corpora still need stem-based IDs to match qrels manifests |
| **Suggested Action** | Keep a shared in-memory document/index layer, but let each loader define its own canonical document ID policy |
| **Applies To** | `src/search.rs`, future hybrid reranking, benchmark corpus loaders |
| **Linked Knowledge IDs** | 1vzLal000 |
| **Score** | 0.88 |
| **Confidence** | 0.95 |
| **Applied** | yes |

