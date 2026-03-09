# Evaluation Guide

Sift uses standard Information Retrieval (IR) evaluation datasets to ensure search quality doesn't degrade over time.

## Concepts

1.  **Raw Dataset:** The original dataset files (e.g., SciFact `corpus.jsonl`).
2.  **Materialized Corpus:** A directory of individual text files derived from the raw dataset. This is what `sift` searches during benchmarks to simulate a real-world local project structure.
3.  **Qrels:** "Query Relevance" judgements. A file mapping query IDs to the correct document IDs.

## The SciFact Dataset

Currently, Sift primarily uses the SciFact dataset for evaluation.

### Download
Downloads the dataset from HuggingFace to `~/.cache/sift/eval/scifact`.

```bash
# Via just
just eval download-scifact

# Via direct CLI
sift eval download scifact
```

### Materialize
Converts the JSONL corpus into a directory of `.txt` files at `~/.cache/sift/eval/scifact-files`. This is necessary for `sift` to walk the "project" and index it normally.

```bash
# Via just
just eval materialize-scifact

# Via direct CLI
sift eval materialize scifact
```

## Creating New Evaluations

To add support for a new dataset:
1.  Implement a loader in `src/eval.rs`.
2.  Update the `Dataset` enum in `src/main.rs`.
3.  Add corresponding recipes to `.justfiles/eval.just`.

## Validation via Verbose Mode

If materialization is slow, you can use verbose mode to trace the extraction process:

```bash
sift eval materialize scifact -v
```
