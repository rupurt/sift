# Architecture

Sift is designed using **Domain-Driven Design (DDD)** and **Hexagonal Architecture (Ports and Adapters)** principles. This ensures that the core search logic is isolated from the CLI interface, benchmark runners, and external libraries.

## Core Tenets

1. **Domain Isolation:** The core search strategy logic does not know about the terminal, CLI arguments, or HTTP clients.
2. **Composable Pipeline:** Search is not a monolithic algorithm. It is an explicit pipeline of `Expansion -> Retrieval -> Fusion -> Reranking`.
3. **Pluggable Adapters:** BM25, Candle-based dense vectors, and phrase matching are implemented as adapters fulfilling domain traits.
4. **Pure Rust:** Sift is a pure-Rust application, with no external C++ or database dependencies.
5. **Hybrid IR System:** `sift` is a Hybrid Information Retrieval (IR) system, bridging the "vocabulary gap" between queries and documents by combining lexical matching, semantic embeddings, and LLM-driven intent analysis.

## Intent-Driven Retrieval

Sift does not just match keywords; it attempts to understand and expand the user's **intent**. Before the retrieval stage begins, the query is passed through an **Expansion Pipeline** that uses a local LLM (Qwen2.5-0.5B) to bridge the "vocabulary gap"—the difference between how a user asks a question and how the code or documentation is written.

- **Explicit Intent:** Users can provide direct context via the `--intent` flag. This bypasses automated inference and guides the search directly.
- **Hypothetical Document Embeddings (HyDE):** For complex informational queries, Sift generates a hypothetical technical answer and uses its embedding for vector retrieval. This often finds better matches than the query alone.
- **SPLADE-style Expansion:** Sift predicts additional technical terms that are semantically related to the query (e.g., "auth" expands to "login, session, token") to increase keyword recall.
- **Intent Classification:** Sift categorizes the query (e.g., BUGFIX vs. NAVIGATION) and adds intent-specific keywords to the retrieval variants.

## Execution Model

### Optimized Retrieval Pipeline
Sift is designed for maximum throughput on local hardware by minimizing I/O overhead and maximizing CPU efficiency:
1. **Mapped I/O:** Uses `mmap` for reading document blobs from the global cache, allowing the OS to handle paging and reducing system call overhead.
2. **SIMD Acceleration:** The core `dot_product` calculation for vector similarity is optimized using architecture-specific SIMD instructions (via the `wide` crate), achieving up to 7x speedup over scalar iterators.
3. **Query Embedding Cache:** A session-level cache eliminates redundant neural network inference for identical queries during complex search strategies or batch evaluations.
4. **Memory Optimization:** Critical scoring paths use pre-allocated buffers and zero-copy references to minimize heap allocations and pressure on the Rust allocator.

## Observability & Performance

### Structured Telemetry
Sift uses the `tracing` crate to provide a detailed view of search execution.
- **Spans:** Major search phases (Expansion, Retrieval, Fusion, Reranking) are wrapped in spans, enabling waterfall visualization of latency.
- **Cache Telemetry:** The `Telemetry` module uses atomic counters to track the effectiveness of the asset pipeline.

### Performance Guardrails
- **Micro-benchmarks:** Crucial low-level functions (e.g., `tokenize`, `dot_product`) are protected by `criterion` benchmarks in the `benches/` directory.
- **Flamegraphs:** Integration with `cargo-flamegraph` allows for deep inspection of CPU bottlenecks.

## Hexagonal Boundaries

### 1. The Domain (`src/search/domain.rs`)

The domain defines the vocabulary of the search process.

- **Expansion Pipeline:**
  - `HydeStrategy`: Generates a hypothetical technical answer to bridge semantic gaps.
  - `SpladeStrategy`: Predicts semantically related technical terms for broader recall.
  - `ClassifiedStrategy`: Categorizes technical intent and adds specific keyword variants.
- **Models:**
  - `Document`: A parsed file with extracted text, length, and content segments.
  - `Candidate` & `CandidateList`: The scored outputs from retrievers and the fuser.
  - `SearchPlan`: A named preset that defines exactly which policies to use for each phase of the pipeline.
- **Ports (Traits):**
  - `Expander`: Takes a query and produces `QueryVariant`s.
  - `Retriever`: Takes query variants and scores the `PreparedCorpus` to produce a `CandidateList`.
  - `Fuser`: Takes multiple `CandidateList`s (e.g., from BM25 and Vector) and merges them into a single ranked list.
  - `Reranker`: Takes the fused list and applies a final pass to finalize the order.

### 2. The Modular Engine (`src/search/engine.rs`)

Sift has evolved from a hardcoded search pipeline into a modular **Graph IR and Execution Engine**. The search logic is decoupled into four foundational traits, allowing for pluggable components and universal retrieval.

- **SearchEngine (The Orchestrator):** The top-level interface that binds IR, Execution, and Storage. It provides a unified `.search()` entry point.
- **SearchIR (The Graph Compiler):** Translates a user query and options into a formal Intermediate Representation (IR) of the search plan. This allows for optimization passes and dynamic graph generation.
- **SearchExecution (The Runtime):** Orchestrates the traversal of the IR graph. Standard execution follows the `Expansion -> Retrieval -> Fusion -> Rerank` sequence.
- **SearchStorage (The Persistence Layer):** Abstracts the document corpus and indices. This allows Sift to be decoupled from the local filesystem, enabling retrieval from S3, databases, or remote APIs.

#### GenericEngine & Factory
The `GenericEngine<IR, Exec, Storage>` is the concrete implementation used by the CLI. The `EngineFactory` streamlines the creation of these engines, handling the registration of standard retrievers, fusers, and LLM-backed expanders.

### 3. The Adapters (`src/search/adapters/`)

Adapters are the concrete implementations of the domain ports. 

- `Bm25Retriever`: Uses the in-memory term-frequency index to score documents.
- `PhraseRetriever`: Performs exact string matching.
- `SegmentVectorRetriever`: Uses the `candle` machine learning framework to embed queries and document segments.
- `QwenReranker`: Uses a local LLM (Qwen2.5) to perform deep semantic relevance scoring of top candidates.
- `PositionAwareReranker`: Applies structural heuristics (filename/heading matches) to boost relevance.
- `RrfFuser`: Implements Reciprocal Rank Fusion to balance scores between different retrieval methods.

## The Incremental File Cache (`src/cache/`)

Sift employs a Zig-inspired incremental caching system to make repeat searches nearly instant. It operates as an asset pipeline in the standard system cache directory (e.g., `~/.cache/sift/` on Linux).

### 1. The Metadata Store (Manifests)
Located in the `manifests/` sub-directory, manifests are keyed by the hash of the absolute path being searched. They map filesystem heuristics (`inode`, `mtime`, `size`, `relative_path`) to a BLAKE3 content hash.

### 2. The Content-Addressable Blob Store (CAS)
Located in the `blobs/` sub-directory, this stores binary serialized `Document` representations.
- **Fully Processed Assets:** Each blob contains the extracted text, pre-computed term frequencies for lexical search, and pre-computed dense vector embeddings for semantic search. This allows `sift` to perform search at the speed of a dot-product without re-running neural network inference on subsequent queries.
- **Atomic Writes:** New blobs are written to a `.tmp` file and then renamed for atomicity.
- **Global Deduplication:** Identical files across different projects only occupy a single entry in the blob store.

### 3. Score Interpretation Model
Sift uses a multi-retriever pipeline with Reciprocal Rank Fusion (RRF). Scores are interpreted based on their proximity to the theoretical maximum for a given search plan:
- **High Confidence (Green):** The result was ranked highly across most or all retrievers.
- **Medium Confidence (Yellow):** The result appeared in at least one retriever with a strong rank, or multiple with lower ranks.
- **Low Confidence (Red):** The result has minimal signal across the retrievers.

## Extraction Pipeline (`src/extract.rs`, `src/segment.rs`)

Files are converted into standard `Document` objects before search begins.

- **Format Agnostic:** PDF, HTML, Markdown, and OOXML files are all parsed into plain text.
- **Structure Aware:** Documents are broken into `Segment`s (e.g., "Page 1", "Section 2") to allow localized vector embeddings and precise snippet extraction.
