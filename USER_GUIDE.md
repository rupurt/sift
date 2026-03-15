# User Guide

Welcome to **Sift**, the lightning-fast, standalone hybrid search engine for local document retrieval. This guide will walk you through everything from your first search to mastering advanced retrieval strategies.

## Introduction

Sift was built to bridge the "vocabulary gap" in technical documentation and code. It combines the speed of traditional lexical search (BM25) with the deep understanding of semantic vector search and LLM-based reranking.

## Getting Started

### Installation

The fastest way to install Sift on macOS or Linux is via Homebrew:

```bash
brew tap rupurt/homebrew-tap
brew install sift
```

For other platforms and installation methods, see the **[README.md](README.md#installation)**.

### Your First Search

Point Sift at any directory and give it a query:

```bash
sift search ./my-project "how do I handle authentication?"
```

On the first run, Sift will extract and index your documents. Subsequent searches will be near-instant.

## Core Concepts

### Hybrid Search
Sift doesn't just look for exact words. It uses **Hybrid Information Retrieval (IR)**:
1.  **Lexical (BM25):** Finds exact keyword matches.
2.  **Semantic (Vector):** Finds related concepts using local machine learning models.
3.  **Phrase:** High-precision exact string matching.

### Intent-Driven Expansion
Sift can use a local LLM to expand your query before searching. This helps find documents that don't share the same exact words as your query but do share the same meaning.

## Advanced Usage

### Choosing a Strategy
Sift comes with several built-in strategy presets. Use the `--strategy` flag to switch between them:

-   `page-index-hybrid` (Default): The best balance of quality and speed.
-   `page-index-llm`: Uses a local LLM for the highest precision reranking.
-   `bm25`: Extremely fast, keyword-only search.
-   `vector`: Pure semantic search.

```bash
sift search --strategy page-index-llm "refactor core engine"
```

### Providing Intent
If you know exactly what you are looking for, use the `--intent` flag to guide the search:

```bash
sift search --intent "I am looking for the trait definitions" "engine"
```

### Filtering and Overrides
You can manually override components of any search:

```bash
sift search --retrievers bm25 --limit 5 "query"
```

## Configuration

Sift is "zero-config" by default, but you can customize its behavior using a `sift.toml` file.

-   **Change Models:** Use a different embedding or reranking model from Hugging Face.
-   **Ignore Files:** Create a `.siftignore` file to skip large or irrelevant directories.

See the **[Configuration Guide](CONFIGURATION.md)** for full details.

## For Developers

### Embedding Sift
Sift is designed to be embedded as a Rust library. It provides a formal **Modular Engine** architecture with traits for IR, Execution, and Storage.

-   **[ARCHITECTURE.md](ARCHITECTURE.md):** Detailed look at the internal design.
-   **[RESEARCH.md](RESEARCH.md):** The vision for the modular engine.
-   **[examples/sift-embed](examples/sift-embed):** A runnable example of how to embed Sift in your own project.

## Troubleshooting

-   **First-run Latency:** The first search in a large directory may take a few seconds as models are downloaded and documents are indexed.
-   **Cache Location:** Sift stores its cache in your standard OS cache directory (e.g., `~/.cache/sift` on Linux). You can clear this at any time to force a full re-index.

---

*For more information, visit the [GitHub Repository](https://github.com/rupurt/sift).*
