# Evaluation & Dataset Guide

Sift includes evaluation harnesses for both the retrieval substrate and the
current agentic search runtime layers.

The important split is:

- **Retrieval evaluations:** Measure single-turn quality and latency for the
  underlying hybrid search plans.
- **Agentic evaluations:** Measure autonomous-planner runs, planned multi-turn
  controller execution, retained-evidence behavior, and comparison against a
  collapsed single-turn baseline.

## Concepts

1. **Raw dataset:** The original dataset files such as SciFact JSONL archives.
2. **Materialized corpus:** A directory of local text files derived from the
   raw dataset. This is what `sift` searches during evaluation.
3. **Qrels:** Query relevance judgments mapping query IDs to relevant documents.
4. **Agentic fixture:** A task fixture with a root task, planned multi-turn
   controller queries, expected per-turn documents, and expected final retained
   evidence.
5. **Collapsed baseline:** A single-turn baseline query created by
   concatenating the planned turn queries for comparison with controller
   execution.

## Dataset Management

Evaluation data is stored in the standard user cache directory, for example
`~/.cache/sift/eval` on Linux or
`~/Library/Caches/com.rupurt.sift/eval` on macOS.

### SciFact

SciFact is the primary retrieval benchmark bundled into the current docs and
tooling.

```bash
just sift dataset download scifact
just sift dataset materialize scifact
```

That materialized corpus is what the retrieval evaluations search.

## Retrieval Evaluations

The retrieval harness exercises the single-turn search substrate that both the
CLI and controller layer reuse.

### Compare All Registered Strategies

`eval all` runs every registered preset and prints a comparative table by
default. Use `--json` if you need structured output. The other evaluation
commands emit structured JSON by default.

```bash
just sift eval all --dataset scifact
just sift eval all --dataset scifact --json
```

Current registered strategies include `lexical`, `bm25`, `vector`, `hybrid`,
`path-hybrid`, `legacy-hybrid`, `page-index-hybrid`, `page-index-llm`,
`page-index-qwen`, `page-index-splade`, `page-index-classified`,
`page-index-jina`, and `page-index-gemma`.

The richer page-index family now evaluates structural fuzzy retrieval as part of
the core substrate: path-aware recall for filename-like intent plus snippet-
bearing fuzzy segment evidence for downstream synthesis consumers.

SciFact remains useful for retrieval quality baselines, but it is text-centric.
If you are evaluating filename-heavy recall or snippet-bearing structural
evidence, pair SciFact with repo-local fixtures or manual query probes that
look like real development workflows.

If a strategy depends on a gated Hugging Face model and `HF_TOKEN` is missing
or lacks access, `eval all` skips that strategy instead of aborting the entire
run.

### Run a Quality Report for One Strategy

`eval quality` emits structured JSON for a single strategy and can optionally
compute deltas against a baseline.

```bash
just sift eval quality --strategy page-index-hybrid --baseline bm25 --dataset scifact
```

This report includes:

- `ndcg_at_10`
- `mrr_at_10`
- `recall_at_10`
- optional deltas versus the supplied baseline
- reactor metrics such as `signal_gain`

### Run a Latency Report for One Strategy

`eval latency` emits structured JSON focused on end-to-end latency.

```bash
just sift eval latency --strategy hybrid --dataset scifact
```

This report includes:

- `prepare_ms`
- `p50_ms`
- `p90_ms`
- `max_ms`
- over-target deltas against the current latency target

### Use a Smaller Query Slice

For faster iteration, especially when testing generative strategies, use
`--query-limit`:

```bash
just sift eval quality --strategy hybrid --dataset scifact --query-limit 10
just sift eval latency --strategy hybrid --dataset scifact --query-limit 10
just sift eval all --dataset scifact --query-limit 10
```

### Structural Retrieval Checks

When you are validating the newer structural lanes, use comparisons that match
the behavior you care about:

- Compare `path-hybrid` against `bm25` for filename- or path-shaped queries.
- Compare `page-index-hybrid` against `hybrid` when snippet-bearing evidence quality matters.
- Keep full `cargo test` coverage in the loop so documentation claims stay anchored to the shipped substrate.

## Agentic Evaluations

`eval agentic` now benchmarks the built-in linear and graph autonomous planner
paths alongside the existing planned controller path. For each fixture it runs:

- the built-in linear autonomous planner from the fixture root task
- the built-in graph autonomous planner from the fixture root task
- the planned multi-turn controller path from the ordered fixture turns
- a collapsed single-turn baseline formed by concatenating the planned queries

```bash
just sift eval agentic \
  --corpus tests/fixtures/agentic-eval/corpus \
  --fixtures tests/fixtures/agentic-eval/fixtures.json \
  --strategy hybrid
```

By default, the baseline strategy for the comparison is `hybrid`, and the
controller/autonomous runtime retains at most one artifact unless you override
`--retained-artifact-limit`.

```bash
just sift eval agentic \
  --corpus tests/fixtures/agentic-eval/corpus \
  --fixtures tests/fixtures/agentic-eval/fixtures.json \
  --strategy page-index-hybrid \
  --retained-artifact-limit 3
```

### What `eval agentic` Measures

The agentic report includes:

- overall `task_success_rate`
- `average_turn_recall`
- `average_final_recall`
- `average_turns`
- `average_prune_actions`
- an `autonomous` block for the linear autonomous run
- a `graph` block for the bounded graph autonomous run
- graph-specific metrics such as frontier expansion cost, merge or prune
  counts, and branch efficiency
- per-task planned-controller traces and per-turn recall
- retained final documents for the planned-controller, linear autonomous, and
  graph autonomous runs
- a comparison block covering linear autonomy, graph autonomy,
  planned-controller, and collapsed-single-turn runs
- latency, turn-count, retained-evidence-efficiency, and graph-metric deltas
  for the graph run versus the linear and baseline runs

Each task comparison records:

- the autonomous root task
- the collapsed baseline query
- expected final documents
- linear autonomous final documents
- graph autonomous final documents
- planned-controller final documents
- baseline final documents
- success/failure for all four runs
- final recall for all four runs
- per-task latency for all four runs
- linear and graph stop reasons
- linear and graph retained-evidence efficiency
- per-task graph metrics such as frontier expansion cost

### What `eval agentic` Does Not Measure Yet

The current agentic harness is useful, but it is intentionally narrow. It does
not yet measure:

- grounded answer synthesis quality or faithfulness
- evidence-retention precision beyond recall/success style metrics
- end-user answer usefulness after retrieval

Those are the next evaluation families to add once the controller grows beyond
bounded local graph retrieval and into answer generation.

## GPU-Backed Evaluation Runs

When you want Candle-backed CUDA builds inside the supported repository setup,
prepend `--cuda` to the `just sift` entrypoint:

```bash
just sift --cuda eval agentic \
  --corpus tests/fixtures/agentic-eval/corpus \
  --fixtures tests/fixtures/agentic-eval/fixtures.json
```

This `--cuda` switch is handled by the `just` recipe, not by the `sift` CLI
itself.

Useful device overrides:

- `SIFT_DENSE_DEVICE=cpu|cuda`
- `SIFT_LLM_DEVICE=cpu|cuda`
- `SIFT_QWEN_DEVICE=cpu|cuda`
- `SIFT_JINA_DEVICE=cpu|cuda`
- `SIFT_GEMMA_DEVICE=cpu|cuda`

Example:

```bash
SIFT_JINA_DEVICE=cuda SIFT_GEMMA_DEVICE=cuda \
  just sift --cuda eval all --dataset scifact
```

## Prompt Optimization

The `optimize` command mutates the prompt templates used for generative
expansion and keeps changes that improve retrieval metrics.

```bash
just sift optimize --dataset scifact --iterations 3
```

This optimization loop currently targets the retrieval substrate, not the
agentic controller layer.

## Profiling

### Criterion Benchmarks

```bash
just bench
```

### Flamegraphs

```bash
just flamegraph all --dataset scifact
```

## Interpreting Results

### Retrieval Metrics

- **nDCG@10:** Ranking quality across relevant results.
- **MRR@10:** How early the first relevant result appears.
- **Recall@10:** Relevant-document coverage in the top 10.
- **Signal Gain:** Improvement over the chosen baseline.

### Agentic Metrics

- **Task success rate:** Fraction of tasks whose expected final documents were satisfied.
- **Average turn recall:** Mean recall across turns that declare expected documents.
- **Average final recall:** Mean recall over final retained evidence.
- **Average turns:** Turn cost of the controller run.
- **Average prune actions:** How often the controller had to evict stale evidence to stay within budget.

### Latency Metrics

- **prepare_ms:** Corpus preparation and environment setup.
- **p50_ms / p90_ms / max_ms:** Median, tail, and worst-case query latency.
- **over_target_ms:** How far each percentile exceeds the current latency target.
