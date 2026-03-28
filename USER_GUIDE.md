# User Guide

Welcome to **Sift**, a standalone local search tool that combines hybrid
retrieval with an emerging agentic controller layer.

The executable is centered on direct single-turn search today. The library and
evaluation harnesses already expose deterministic turn-aware behavior for
controller-style workflows.

## Getting Started

### Installation

The fastest way to install Sift on macOS or Linux is:

```bash
brew tap rupurt/homebrew-tap
brew install sift
```

For other installation methods, see [README.md](README.md#installation).

### Your First Search

```bash
sift search ./my-project "how do I handle authentication?"
```

On first run, Sift extracts and indexes the corpus into its transparent local
cache. Repeated searches reuse those assets.

## Core Ideas

### Hybrid Retrieval

Sift combines:

1. **BM25:** Lexical keyword retrieval.
2. **Phrase matching:** Exact high-precision matches.
3. **Vector retrieval:** Semantic similarity via local embeddings.
4. **Reranking:** Optional semantic or structure-aware reranking.

### Agentic Status

Agentic support in Sift currently shows up in four places:

1. **Intent-aware search:** `--intent` lets you add task context to a search.
2. **Planner-driven CLI search:** `sift search --agent` runs the shared
   autonomous runtime from the executable.
3. **Turn-aware library APIs:** The crate root exposes `search_turn`,
   `search_controller`, `search_autonomous`, and protocol/latent emissions.
4. **Fixture-driven autonomous/controller evals:** `sift eval agentic`
   benchmarks autonomous planner runs, planned multi-turn controller runs, and
   collapsed single-turn baselines.

What is not shipped yet is a general-purpose interactive agent shell or
branching/graph autonomous search.

## Choosing a Strategy

Useful built-in strategies:

- `hybrid`: Default direct-search strategy.
- `page-index-hybrid`: Current richer evaluation champion preset.
- `page-index-llm`: HyDE plus Qwen reranking.
- `bm25`: Fast lexical-only search.
- `vector`: Semantic-only search.

Examples:

```bash
sift search --strategy hybrid "refactor core engine"
sift search --strategy page-index-hybrid "refactor core engine"
sift search --strategy bm25 "service catalog"
```

## Useful Flags

### Intent

```bash
sift search --intent "I am looking for the trait definitions" "engine"
```

### Agent Mode

```bash
sift search --strategy bm25 ./my-project --agent "find the cache invalidation path"
```

Swap to the model-driven planner when you have a local planner profile wired
through the same runtime:

```bash
sift search --strategy hybrid --agent "trace the cache invalidation path" \
  --planner-strategy model-driven --planner-profile local-planner-v1
```

### Manual Pipeline Overrides

```bash
sift search --retrievers bm25,phrase --reranking none --limit 5 "query"
```

### JSON Output

```bash
sift search --json "query"
```

## Configuration

Sift works with no config, but you can customize it through `sift.toml` and
`.siftignore`.

- Change the default search strategy.
- Point Sift at different embedding or reranking models.
- Override generative prompts used by HyDE, SPLADE, and classified expansion.
- Ignore noisy directories such as `target/` or build outputs.

See [CONFIGURATION.md](CONFIGURATION.md) for the full configuration surface.

## Evaluations

Sift ships two evaluation families:

- **Retrieval evals:** `sift eval all`, `sift eval quality`, `sift eval latency`
- **Agentic evals:** `sift eval agentic`

See [EVALUATIONS.md](EVALUATIONS.md) for datasets, metrics, and report shapes.

## For Developers

- [LIBRARY.md](LIBRARY.md): Supported crate-root embedding guide.
- [ARCHITECTURE.md](ARCHITECTURE.md): Internal architecture and execution seams.
- [WORLD.md](WORLD.md): Conceptual model and reactor metaphor.
- [examples/sift-embed](examples/sift-embed): Runnable embedding example.

## Troubleshooting

- **First-run latency:** Initial searches may download models and build cache artifacts.
- **Cache location:** By default Sift uses your OS cache directory, for example `~/.cache/sift` on Linux.
- **Model access:** Some rerankers require gated Hugging Face model access through `HF_TOKEN`.
