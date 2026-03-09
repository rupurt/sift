# Benchmarking Guide

Sift includes a built-in benchmarking harness to measure retrieval quality and latency. These benchmarks are essential for validating architectural changes and strategy improvements.

## Prerequisite: Preparing the Dataset

Before running benchmarks, you must download and materialize an evaluation dataset (e.g., SciFact).

```bash
# Downloads and prepares the SciFact dataset automatically
just bench prepare
```

## Running Benchmarks via `just`

The `just` recipes pass all arguments through to the underlying `sift bench` command.

### 1. Comparative Benchmark (`bench all`)
Runs all available strategies (BM25, Vector, Hybrid, etc.) and compares their metrics.

```bash
just bench all --corpus ".cache/eval/scifact-files" --qrels ".cache/eval/scifact/qrels/test.tsv"
```

### 2. Champion Benchmark (`bench hybrid`)
Runs a comprehensive quality and latency report for the current champion strategy (`page-index-hybrid`) against the `bm25` baseline.

```bash
just bench hybrid --corpus ".cache/eval/scifact-files" --qrels ".cache/eval/scifact/qrels/test.tsv" --queries ".cache/eval/scifact-files/test-queries.tsv"
```

### 3. Baseline Benchmark (`bench baseline`)
Runs a report for the standard `bm25` strategy.

```bash
just bench baseline --corpus ".cache/eval/scifact-files" --qrels ".cache/eval/scifact/qrels/test.tsv" --queries ".cache/eval/scifact-files/test-queries.tsv"
```

---

## Direct CLI Usage

For more control, you can use the `sift bench` subcommand directly.

### Quality Benchmark
Measures retrieval metrics (nDCG, MRR, Recall) using a Qrels file.

```bash
sift bench quality \
  --strategy page-index-hybrid \
  --corpus .cache/eval/scifact-files \
  --qrels .cache/eval/scifact/qrels/test.tsv
```

### Latency Benchmark
Measures p50, p90, and max latency over a set of queries.

```bash
sift bench latency \
  --strategy vector \
  --corpus .cache/eval/scifact-files \
  --queries .cache/eval/scifact-files/test-queries.tsv
```

## Debugging Benchmarks (`--verbose`)

Benchmarks support the standard verbosity flags (`-v`, `-vv`, `-vvv`). You can pass these through `just`:

```bash
just bench all --corpus ".cache/eval/scifact-files" --qrels ".cache/eval/scifact/qrels/test.tsv" -v
```

## Interpreting Results

- **nDCG@10:** Measures ranking quality based on relevance.
- **MRR@10:** Measures how high the first relevant document appears.
- **Recall@10:** Measures the percentage of relevant documents found in the top 10.
- **p50 (ms):** The median search latency.
