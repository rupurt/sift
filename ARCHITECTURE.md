# Architecture

Sift is designed using **Domain-Driven Design (DDD)** and **Hexagonal Architecture (Ports and Adapters)** principles. It has evolved into a modular **Graph IR and Execution Engine** capable of searching both local files and conversational agent turns.

## Core Tenets

1. **Domain Isolation:** The core search strategy logic is decoupled from terminal interfaces, storage backends, and execution runtimes.
2. **Modular Engine:** Search is governed by formal traits (`SearchIR`, `SearchExecution`, `SearchStorage`), enabling pluggable components.
3. **Multi-Modal Emission:** Decouples retrieval from presentation, supporting headless latent embeddings, domain records (Turns), and rendered views.
4. **Pure Rust:** Sift is a pure-Rust application, with no external C++ or database dependencies.
5. **Hybrid IR System:** `sift` bridges the "vocabulary gap" by combining lexical matching, semantic embeddings, and LLM-driven intent analysis.

## The Modular Engine Architecture

The search process is orchestrated by a unified `SearchEngine` interface that binds three specialized layers:

### 1. The Domain (`src/search/domain.rs`)
Defines the vocabulary of retrieval, including `Document` and `AgentTurn` models, and the core trait boundaries (`Expander`, `Retriever`, `Fuser`, `Reranker`).

### 2. SearchIR (The Graph Compiler)
The Intermediate Representation (IR) translates user queries into an executable **Graph of Operations**. This allows for dynamic strategy generation and optimization passes before any data is retrieved.

### 3. SearchExecution (The Runtime)
Orchestrates the traversal of the IR graph. By making execution a trait, Sift can support different runtimes—from a simple sequential pipeline to high-concurrency parallel walks using Rayon or asynchronous task graphs using Tokio.

### 4. SearchStorage (The Persistence Layer)
Abstracts the corpus and indices. This allows Sift to be a **Universal Retrieval Engine**, capable of searching the local filesystem, S3 buckets, databases, or remote conversational logs.

## Agentic IR & Emission

Sift supports searching and surfacing **Agent Turns**—the conversational history of AI agents—capturing the dynamic lineage of logic rather than just static text.

### Emission Strategies
The engine supports multiple emission modes to satisfy different consumers:
- **Latent Emission (`emit_latent`):** Returns raw embedding vectors (Tensors) for handoff to external ranking or search systems.
- **Turn Emission (`emit_turns`):** Returns high-level `AgentTurn` records for conversational interfaces.
- **View Emission (`emit_view`):** Returns rendered, highlighted text for standard CLI display.

## Intent-Driven Retrieval

Sift uses a local LLM (Qwen2.5-0.5B) to understand and expand user intent before retrieval begins:
- **Explicit Intent:** Guides search via the `--intent` flag.
- **HyDE:** Generates hypothetical answers to bridge semantic gaps.
- **SPLADE:** Predicts semantically related technical terms.
- **Classification:** Categorizes queries (e.g., BUGFIX) to add intent-specific keywords.

## The Incremental File Cache (`src/cache/`)

Sift employs a Zig-inspired incremental caching system to make repeat searches nearly instant.

### 1. Metadata Store (Manifests)
Maps filesystem heuristics (`inode`, `mtime`, `size`) to a strong content hash.

### 2. Content-Addressable Blob Store (CAS)
Stores binary serialized assets, including extracted text, term frequencies, and pre-computed dense vector embeddings. This allows search to run at dot-product speeds by bypassing neural network inference on subsequent queries.

## Performance Guardrails

- **SIMD Acceleration:** Optimized `dot_product` calculations via the `wide` crate for 7x speedups.
- **Mapped I/O:** Uses `mmap` for reading document blobs to minimize system call overhead.
- **Query Embedding Cache:** Session-level cache eliminates redundant inference for identical queries.
- **Structured Telemetry:** Uses the `tracing` crate for waterfall visualization of phase latency.
