# Architecture

Sift is designed using **Domain-Driven Design (DDD)** and **Hexagonal Architecture (Ports and Adapters)** principles. It operates as a **High-Energy Information Reactor** capable of searching both local files and conversational agent turns.

## Core Tenets

1. **Information Physics:** Search is viewed as a two-stage process: **Magnetism** (pulling relevant mass into a containment field) and **Fusion** (reacting upon that mass to emit intent-aligned energy).
2. **Modular Reactor:** The engine is governed by formal traits (`SearchIR`, `SearchExecution`, `SearchStorage`), enabling pluggable components.
3. **Multi-Modal Emission:** Decouples retrieval from presentation, supporting headless latent embeddings, domain records (Turns), and rendered views via dedicated **Emission Ports**.
4. **Pure Rust:** Sift is a pure-Rust application, with no external C++ or database dependencies.
5. **Hybrid IR System:** `sift` bridges the "vocabulary gap" by combining lexical matching, semantic embeddings, and LLM-driven intent analysis.

## The Reactor Architecture

The search process is orchestrated by a unified **Reactor** interface (the `SearchEngine` trait) that binds three specialized layers:

### 1. The Domain (`src/search/domain.rs`)
Defines the vocabulary of retrieval, including `Document` and `AgentTurn` models, and the core trait boundaries (`Expander`, `Retriever`, `Fuser`, `Reranker`).

### 2. SearchIR (The Magnetic Field Configuration)
The Intermediate Representation (IR) translates user queries into an executable **Graph of Operations**. This defines the "magnetic field" that pulls raw mass from storage.

### 3. SearchExecution (The Fusion Runtime)
Orchestrates the traversal of the IR graph and the reaction process. By making execution a trait, Sift can support different runtimes—from sequential pipelines to high-concurrency parallel walks using Rayon.

### 4. SearchStorage (The Mass Repository)
Abstracts the corpus and indices. This allows Sift to be a **Universal Retrieval Engine**, capable of searching the local filesystem, S3 buckets, or remote conversational logs.

## Agentic IR & Emission Ports

Sift supports searching and surfacing **Agent Turns**—the conversational history of AI agents—capturing the dynamic lineage of logic rather than just static text.

### Emission Modes
The reactor features configurable ports to bleed off different types of energy:
- **Latent Emission:** Raw embedding vectors (Tensors) for handoff to external systems.
- **Protocol Emission:** Structured domain records (e.g., `AgentTurn`) for agentic consumers.
- **Visual Emission:** Rendered, highlighted text for standard human-facing CLI display.

## Intent-Driven Retrieval (Catalysis)

Sift uses local LLMs (Qwen 2.5, Gemma 3) to understand and expand user intent, acting as a catalyst for the retrieval reaction:
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

## Adapters (`src/search/adapters/`)

Adapters implement the core search traits, enabling pluggable behavior across the reactor:

### 1. Expansion (`Expander`)
- **LlmExpander:** Uses local LLMs for generative expansion (HyDE, SPLADE, Classified).
- **SynonymExpander:** Rule-based synonym matching.

### 2. Retrieval (`Retriever`)
- **Bm25Retriever:** Lexical scoring using the BM25 algorithm.
- **PhraseRetriever:** High-precision exact phrase matching.
- **SegmentVectorRetriever:** Semantic scoring via dense vector embeddings.

### 3. Fusion (`Fuser`)
- **RrfFuser:** Combines multiple candidate lists using Reciprocal Rank Fusion (RRF).

### 4. Reranking (`Reranker`)
- **PositionAwareReranker:** Applies structural bonuses (filename, heading matches).
- **QwenReranker:** Deep semantic reranking using the Qwen 2.5 family.
- **GemmaReranker:** Deep semantic reranking using the Gemma 3 family.
- **JinaReranker:** Integration with Jina Reranker v3 for high-precision cross-encoding.
