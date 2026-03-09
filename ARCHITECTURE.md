# Architecture

Sift is designed using **Domain-Driven Design (DDD)** and **Hexagonal Architecture (Ports and Adapters)** principles. This ensures that the core search logic is isolated from the CLI interface, benchmark runners, and external libraries.

## Core Tenets

1. **Domain Isolation:** The core search strategy logic does not know about the terminal, CLI arguments, or HTTP clients.
2. **Composable Pipeline:** Search is not a monolithic algorithm. It is an explicit pipeline of `Expansion -> Retrieval -> Fusion -> Reranking`.
3. **Pluggable Adapters:** BM25, Candle-based dense vectors, and phrase matching are implemented as adapters fulfilling domain traits.
4. **Pure Rust:** Sift is a pure-Rust application, with no external C++ or database dependencies.

## Execution Model

### Parallel Processing (`rayon`)
Sift leverages multi-core parallelism via the `rayon` crate during two critical phases:
1. **Corpus Loading:** Files are extracted, segmented, and vectorized in parallel across all available CPU cores.
2. **Evaluation Execution:** Multiple queries are evaluated concurrently during quality and latency testing.

This parallel execution, combined with the heuristic cache, allows `sift` to scale to large repositories while maintaining sub-second search latency.

## Observability & Performance

### Structured Telemetry
Sift uses the `tracing` crate to provide a detailed view of search execution.
- **Spans:** Major search phases (Expansion, Retrieval, Fusion, Reranking) are wrapped in spans, enabling waterfall visualization of latency.
- **Cache Telemetry:** The `Telemetry` module uses atomic counters to track the effectiveness of the asset pipeline across multi-threaded operations.

### Performance Guardrails
- **Micro-benchmarks:** Crucial low-level functions (e.g., `tokenize`) are protected by `criterion` benchmarks in the `benches/` directory.
- **Flamegraphs:** Integration with `cargo-flamegraph` allows for deep inspection of CPU bottlenecks.

## Hexagonal Boundaries

### 1. The Domain (`src/search/domain.rs`)

The domain defines the vocabulary of the search process.

- **Models:**
  - `Document`: A parsed file with extracted text, length, and content segments.
  - `Candidate` & `CandidateList`: The scored outputs from retrievers and the fuser.
  - `SearchPlan`: A named preset that defines exactly which policies to use for each phase of the pipeline.
- **Ports (Traits):**
  - `Expander`: Takes a query and produces `QueryVariant`s.
  - `Retriever`: Takes query variants and scores the `PreparedCorpus` to produce a `CandidateList`.
  - `Fuser`: Takes multiple `CandidateList`s (e.g., from BM25 and Vector) and merges them into a single ranked list.
  - `Reranker`: Takes the fused list and applies a final pass to finalize the order.

### 2. The Application Service (`src/search/application.rs`)

The `SearchService` is the orchestrator. It holds registries of the concrete adapters and executes a `SearchPlan` by passing data through the ports.

### 3. The Adapters (`src/search/adapters/`)

Adapters are the concrete implementations of the domain ports. 

- `Bm25Retriever`: Uses the in-memory term-frequency index to score documents.
- `PhraseRetriever`: Performs exact string matching.
- `SegmentVectorRetriever`: Uses the `candle` machine learning framework to embed queries and document segments.
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
