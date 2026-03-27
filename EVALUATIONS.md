# Evaluation & Dataset Guide

Sift includes a built-in evaluation harness to measure retrieval quality and
latency. These evaluations are essential for validating architectural changes
and strategy improvements.

Today this harness measures the single-turn hybrid retrieval core. As Sift
evolves into a hybrid and agentic search tool, these evaluations remain the
ground-truth gauges for the retrieval substrate that an agentic controller will
reuse.

## Concepts

1.  **Raw Dataset:** The original dataset files (e.g., SciFact `corpus.jsonl`).
2.  **Materialized Corpus:** A directory of individual text files derived from the raw dataset. This is what `sift` searches during evaluations to simulate a real-world local project structure.
3.  **Qrels:** "Query Relevance" judgements. A file mapping query IDs to the correct document IDs.
4.  **Trajectory:** A planned multi-turn search trace for agentic evaluation, including ordered retrieval turns, retained evidence, and context edits.

---

## Dataset Management

Before running evaluations, you must download and materialize an evaluation dataset (e.g., SciFact). Sift automates this via the `dataset` subcommand.

All files are stored in your standard user cache directory (e.g., `~/.cache/sift/eval` on Linux or `~/Library/Caches/com.rupurt.sift/eval` on macOS).

### 1. The SciFact Dataset
SciFact is our primary dataset for local retrieval testing.

```bash
# Downloads and prepares the SciFact dataset automatically
# Files will be placed in your user cache directory
just sift dataset prepare
```

### 2. Manual Dataset Commands
You can also run the download and materialization steps manually:

```bash
# Download
just sift dataset download scifact

# Materialize (JSONL -> individual .txt files)
just sift dataset materialize scifact
```

---

## Running Evaluations

The `eval` subcommand is used to measure search performance for the current
retrieval runtime.

### 1. Comparative Evaluation (`eval all`)
Runs all available strategies (BM25, Vector, Hybrid, etc.) and compares their metrics.

**Note:** Comparative evaluations are significantly accelerated by the **Query Embedding Cache**. Once a query is embedded for the first strategy, all subsequent strategies will reuse that embedding, reducing total runtime by hundreds of milliseconds per query.

**Note:** Some comparison strategies rely on gated Hugging Face models. When those models are inaccessible because `HF_TOKEN` is unset or lacks access, `eval all` skips the affected strategies instead of aborting the entire benchmark run.

```bash
just sift eval all --dataset scifact
```

### 2. Champion Evaluation (`eval hybrid`)
Runs a comprehensive quality and latency report for the current champion strategy (`page-index-hybrid`) against the `bm25` baseline.

```bash
just sift eval hybrid --dataset scifact
```

### 3. Baseline Evaluation (`eval baseline`)
Runs a report for the standard `bm25` strategy.

```bash
just sift eval baseline --dataset scifact
```

### 4. Running a Subset (`--query-limit`)
For large datasets like SciFact, you can limit the number of queries evaluated to speed up the development cycle:

```bash
# Evaluate only the first 5 queries
just sift eval all --dataset scifact --query-limit 5
```

### 5. Intent-Driven Evaluation
Sift allows you to compare different query expansion strategies to see which one best captures user intent for a given dataset.

- **`page-index-splade`**: Measures the quality of generative expansion terms.
- **`page-index-classified`**: Measures the quality of intent-based classification.
- **`page-index-llm`**: Measures the quality of full HyDE expansion combined with LLM reranking.

### 6. Agentic Evaluation (`eval agentic`)
Sift now includes a repo-local harness for planned multi-turn evaluation. These fixtures describe explicit turn sequences and expected documents so the current controller can be measured deterministically without requiring a hosted service or a learned planner.

```bash
just sift eval agentic \
  --corpus tests/fixtures/agentic-eval/corpus \
  --fixtures tests/fixtures/agentic-eval/fixtures.json \
  --strategy bm25
```

The agentic report captures:

- End-to-end task success over planned multi-turn trajectories.
- Per-turn document recall against expected documents.
- Average turns executed per task.
- Context pruning actions emitted by the controller trace.

This harness is intentionally fixture-driven: it evaluates the current local controller and trace contracts, not an autonomous query-decomposition model. Comparative benchmarking against the hybrid champion remains a separate step.

When you want GPU acceleration for Candle-backed models in the supported Nix environment, prepend `--cuda` to the `just sift` entrypoint so the binary is built with the CUDA feature enabled:

```bash
just sift --cuda eval agentic \
  --corpus tests/fixtures/agentic-eval/corpus \
  --fixtures tests/fixtures/agentic-eval/fixtures.json
```

This `--cuda` switch is handled by the `just` recipe, not by the `sift` CLI itself.
By default, that recipe keeps the dense embedder on CPU (`SIFT_DENSE_DEVICE=cpu`) so local GPUs can be reserved for Qwen/Jina/Gemma during evals. If you want dense embeddings on CUDA too, override it explicitly with `SIFT_DENSE_DEVICE=cuda just sift --cuda ...`.

---

## Prompt Optimization

The `optimize` subcommand is used to auto-tune the system prompts used for generative expansion. It uses an LLM to iteratively mutate prompts and measures their impact on **Signal Gain** using the evaluation harness.

### How it Works
1.  **Baseline:** It runs a baseline evaluation of a strategy (e.g., `page-index-splade`) to establish current performance.
2.  **Mutation:** It prompts an LLM to generate a more effective variation of the system prompt.
3.  **Evaluation:** It re-runs the evaluation using the new prompt.
4.  **Selection:** If the new prompt improves Signal Gain, it is kept; otherwise, it is discarded.
5.  **Persistence:** The final optimized prompts are saved to your local `sift.toml`.

### Running the Optimizer

```bash
# Optimize all generative prompts (3 iterations each)
just sift optimize --dataset scifact --iterations 3
```

**Note:** Optimization is an expensive operation as it requires multiple LLM calls and multiple evaluation passes. It is recommended to use a `--query-limit` during initial testing.

---

## Performance Profiling

### Micro-benchmarks (`criterion`)
We use `criterion` for high-precision measurement of hot-path functions like tokenization and scoring.

```bash
just bench
```

### Flamegraphs
Identify CPU bottlenecks and visualize where time is being spent in the search pipeline.

```bash
# Requires cargo-flamegraph installed.
# Note: This command uses sudo to access kernel performance counters.
just flamegraph all --dataset scifact
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
