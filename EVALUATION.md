# Evaluation & Dataset Guide

Sift includes a built-in evaluation harness to measure retrieval quality and latency. These evaluations are essential for validating architectural changes and strategy improvements.

## Concepts

1.  **Raw Dataset:** The original dataset files (e.g., SciFact `corpus.jsonl`).
2.  **Materialized Corpus:** A directory of individual text files derived from the raw dataset. This is what `sift` searches during evaluations to simulate a real-world local project structure.
3.  **Qrels:** "Query Relevance" judgements. A file mapping query IDs to the correct document IDs.

---

## Dataset Management

Before running evaluations, you must download and materialize an evaluation dataset (e.g., SciFact). Sift automates this via the `dataset` subcommand.

All files are stored in your standard user cache directory (e.g., `~/.cache/sift/eval` on Linux).

### 1. The SciFact Dataset
SciFact is our primary dataset for local retrieval testing.

```bash
# Downloads and prepares the SciFact dataset automatically
# Files will be placed in $HOME/.cache/sift/eval/
just dataset prepare
```

### 2. Manual Dataset Commands
You can also run the download and materialization steps manually:

```bash
# Download
just dataset download scifact

# Materialize (JSONL -> individual .txt files)
just dataset materialize scifact
```

---

## Running Evaluations

The `eval` subcommand (and the `just eval` module) is used to measure search performance.

### 1. Comparative Evaluation (`eval all`)
Runs all available strategies (BM25, Vector, Hybrid, etc.) and compares their metrics.

```bash
just eval all --corpus $HOME/.cache/sift/eval/scifact-files --qrels $HOME/.cache/sift/eval/scifact/qrels/test.tsv
```

### 2. Champion Evaluation (`eval hybrid`)
Runs a comprehensive quality and latency report for the current champion strategy (`page-index-hybrid`) against the `bm25` baseline.

```bash
just eval hybrid --corpus $HOME/.cache/sift/eval/scifact-files --qrels $HOME/.cache/sift/eval/scifact/qrels/test.tsv --queries $HOME/.cache/sift/eval/scifact-files/test-queries.tsv
```

### 3. Baseline Evaluation (`eval baseline`)
Runs a report for the standard `bm25` strategy.

```bash
just eval baseline --corpus $HOME/.cache/sift/eval/scifact-files --qrels $HOME/.cache/sift/eval/scifact/qrels/test.tsv --queries $HOME/.cache/sift/eval/scifact-files/test-queries.tsv
```

### 4. Running a Subset (`--query-limit`)
For large datasets like SciFact, you can limit the number of queries evaluated to speed up the development cycle:

```bash
# Evaluate only the first 5 queries
just eval all --corpus $HOME/.cache/sift/eval/scifact-files --qrels $HOME/.cache/sift/eval/scifact/qrels/test.tsv --query-limit 5
```

---

## Performance Profiling

### Micro-benchmarks (`criterion`)
We use `criterion` for high-precision measurement of hot-path functions like tokenization and scoring.

```bash
just eval-micro
```

### Flamegraphs
Identify CPU bottlenecks and visualize where time is being spent in the search pipeline.

```bash
# Requires cargo-flamegraph installed
just eval-flamegraph all --corpus $HOME/.cache/sift/eval/scifact-files --qrels $HOME/.cache/sift/eval/scifact/qrels/test.tsv
```

---

## Interpreting Results

- **nDCG@10:** Measures ranking quality based on relevance.
- **MRR@10:** Measures how high the first relevant document appears.
- **Recall@10:** Measures the percentage of relevant documents found in the top 10.
- **p50 (ms):** The median search latency.
- **Cache Hits:** Shows the percentage of files/segments that hit the heuristic, blob, and embedding caches respectively (e.g., `100/100/100%`).

### Result Highlighting
The evaluation table uses strict color-coding to identify performance outliers:
- **Bold Green:** The **best** performer in that column (highest quality or lowest latency).
- **Bold Red:** The **worst** performer in that column.
- **Bold Yellow/Orange:** The value closest to the **median** of the set.
- **Uncolored:** Average "middle" performers.
