# sift

[![CI](https://github.com/rupurt/sift/actions/workflows/ci.yml/badge.svg)](https://github.com/rupurt/sift/actions/workflows/ci.yml)
[![Planning Board](https://img.shields.io/badge/Keel-Board-blue)](.keel/README.md)
[![Release Process](https://img.shields.io/badge/Release-Process-green)](RELEASE.md)

`sift` is a standalone Rust CLI for local document retrieval in agentic
workflows. It searches raw local corpora without a long-running daemon, uses a 
composable search strategy architecture, and employs a Zig-style heuristic 
caching system for near-instant repeated queries.

The core idea is simple: point `sift` at a directory, extract text on demand,
and run a layered search pipeline (Expansion, Retrieval, Fusion, Reranking). 
There is no external database, no daemon, and no background indexing service.

For the project background and design rationale, read the introductory post:
[`Sift: Local Hybrid Search Without the Infrastructure Tax`](https://www.alexdk.com/blog/introducing-sift).

## Current Contract

- **Single Rust Binary:** No external database, daemon, or long-running service.
- **Pure-Rust Toolchain:** No C++ dependencies, enabling easy static binary distribution.
- **High Performance:** SIMD-accelerated scoring and memory-mapped I/O for 
  ultra-fast retrieval on large corpora.
- **Default Strategy:** Uses the `page-index-hybrid` champion preset (Lexical + Semantic).
- **Layered Pipeline:** Query Expansion -> Retrieval -> Fusion -> Reranking.
- **Heuristic Incremental Caching:** Uses `mtime`, `inode`, and `size` to bypass 
  extraction and hashing for unchanged files.
- **Fully Processed Assets:** Cache blobs contain text, term stats, and pre-computed
  dense vector embeddings, enabling search at dot-product speeds.
- **Inference & Reranking:** Runs locally through Candle with support for 
  both dense embeddings and advanced LLM reranking (e.g., Qwen2.5).
- **Supported Inputs:** Text, HTML, PDF, and OOXML files (`.docx`, `.xlsx`, `.pptx`).

## Installation

### Homebrew (macOS and Linux)

```bash
brew tap rupurt/homebrew-tap
brew install sift
```

### One-liner Install (macOS and Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/rupurt/sift/releases/latest/download/sift-installer.sh | sh
```

### Manual Download

Download the latest pre-built binaries and installers for your platform from the [GitHub Releases](https://github.com/rupurt/sift/releases) page. We provide:
- **Linux:** `.tar.gz` archives plus the cross-platform shell installer
- **macOS:** `.tar.gz` archives plus the cross-platform shell installer
- **Windows:** `.zip` archives, `.msi`, and the PowerShell installer

## How Sift Works

At runtime, `sift` orchestrates a high-performance asset pipeline:

```mermaid
flowchart TD
  A[CLI parse] --> B[Resolve PATH + QUERY]
  B --> C[Directory Walk]
  
  subgraph Pipeline ["Optimized Asset Pipeline"]
    C1{Heuristic Hit?} -->|Yes| C2[Mapped I/O: Load Document Blob]
    C1 -->|No| C3[BLAKE3 Hash + Check CAS]
    C3 -->|Hit| C4[Update Manifest + Load Blob]
    C3 -->|Miss| C5[Extract + Embed + Save Blob]
  end

  C --> C1
  C2 --> D[Build Transient Corpus]
  C4 --> D
  C5 --> D

  subgraph Strategy ["Layered Search Strategy"]
    D --> E[Query Expansion]
    E --> F{Retrievers}
    F -->|Lexical| F1[BM25]
    F -->|Exact| F2[Phrase]
    F -->|Semantic| F3[Vector - SIMD Dot Product]
    
    subgraph QueryCache ["Query Embedding Cache"]
      F3
    end

    F1 --> G[Reciprocal Rank Fusion]
    F2 --> G
    F3 --> G
    G --> H{Reranking}
    H -->|Basic| H1[Position-Aware]
    H -->|Advanced| H2[LLM - Deep Semantic Pass]
  end

  H1 --> I[Colorized CLI Output]
  H2 --> I
```

## Performance & Scalability

`sift` is optimized for speed without sacrificing its stateless UX:
- **Zero-Inference Search:** On repeated queries, neural network inference is bypassed entirely by loading pre-computed embeddings and reusing query embeddings from the session cache.
- **SIMD Acceleration:** Vector similarity calculations use hardware-specific SIMD instructions for a ~7x speedup over standard scalar implementations.
- **Mapped I/O:** Document retrieval from the global cache uses `mmap` to minimize system call overhead and leverage OS-level paging.
- **Fast Path Heuristics:** Filesystem metadata checks happen in microseconds, allowing `sift` to skip hashing for unchanged files.

## Search Examples

The default strategy (champion alias `page-index-hybrid`) is used automatically:

```bash
sift search tests/fixtures/rich-docs "architecture decision"
```

Force a specific strategy or override components:

```bash
# Lexical only (no vectors)
sift search --strategy page-index "service catalog"

# Semantic only (vectors)
sift search --strategy vector "architecture"

# Custom mix
sift search --retrievers bm25,vector --reranking none "query"
```

### Verbose Mode
Trace the pipeline and timings at different levels:
- `-v`: Phase timings (Loading, Retrieval, etc.)
- `-vv`: Detailed retriever timings and cache hit/miss traces.
- `-vvv`: Granular internal scoring data.

## Configuration & Customization

- **[CONFIGURATION.md](CONFIGURATION.md):** Guide to `sift.toml`, available strategies, and environment variables.
- **[EVALUATION.md](EVALUATION.md):** How to manage datasets and run quality/latency evaluations.
- **[ARCHITECTURE.md](ARCHITECTURE.md):** Deep dive into the hexagonal design and asset pipeline.

## License

This project is licensed under the [MIT License](LICENSE).
