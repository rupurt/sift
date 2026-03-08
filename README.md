# sift 🔍

**Standalone hybrid search (BM25 + Vector) for lightning-fast document retrieval in agentic workflows—no database, just a single binary.**

`sift` is a high-performance search utility designed specifically for agentic coding harnesses and modern CLI environments. It provides the sophistication of hybrid ranking (BM25 keyword search + Vector semantic search) without the overhead of external databases, running LLMs, or complex ingestion pipelines.

## 🚀 Key Features

- **Hybrid Ranking:** Combines the precision of BM25 keyword matching with the semantic depth of Vector embeddings.
- **Zero Database:** No SQLite, no Chroma, no Pinecone. It uses in-memory, disk-backed indices that live alongside your code.
- **Single Binary:** A self-contained Rust executable. Drop it into any environment and start searching.
- **Agent-Optimized:** Built-in JSON output and context-window-friendly snippets designed for ingestion by LLM agents.
- **Ultra-Fast:** Powered by `zvec` for vectors and a high-performance Rust BM25 implementation.

## 🛠️ Tech Stack

- **Vector Engine:** [zvec](https://github.com/alibaba/zvec) — Alibaba's "SQLite of Vector Databases," an in-process, high-performance vector search engine.
- **Keyword Search:** [bm25](https://crates.io/crates/bm25) — A native Rust implementation of the BM25 ranking function, optimized for in-memory and disk-backed retrieval.
- **Embeddings:** Support for local lightweight embedding models (via `candle`) or fast API-based embeddings.

## 📦 Installation

```bash
cargo install sift-search
```

## 📖 Usage

### Indexing a directory
```bash
sift index ./docs
```

### Searching
```bash
# Basic keyword search
sift search "authentication logic"

# Hybrid search (Vector + BM25)
sift search --hybrid "how do I reset the user password?"

# Agent-friendly JSON output
sift search --json "api endpoints" | jq .
```

## 🧠 Why Sift?

Existing tools often require you to run a separate vector database or a heavy local LLM just to get decent document retrieval. `sift` takes the opposite approach: it’s a lightweight tool that focuses on **ranking quality** and **developer experience**. 

It’s the "ripgrep" of semantic search—fast, portable, and focused.

## 📜 License

MIT / Apache 2.0
