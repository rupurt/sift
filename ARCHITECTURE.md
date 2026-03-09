# Architecture

Sift is designed using **Domain-Driven Design (DDD)** and **Hexagonal Architecture (Ports and Adapters)** principles. This ensures that the core search logic is isolated from the CLI interface, benchmark runners, and external libraries.

## Core Tenets

1. **Domain Isolation:** The core search strategy logic does not know about the terminal, CLI arguments, or HTTP clients.
2. **Composable Pipeline:** Search is not a monolithic algorithm. It is an explicit pipeline of `Expansion -> Retrieval -> Fusion -> Reranking`.
3. **Pluggable Adapters:** BM25, Candle-based dense vectors, and phrase matching are implemented as adapters fulfilling domain traits.

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
  - `Reranker`: Takes the fused list and applies a final cross-encoder or LLM-based re-scoring pass.

### 2. The Application Service (`src/search/application.rs`)

The `SearchService` is the orchestrator. It holds registries of the concrete adapters and executes a `SearchPlan` by passing data through the ports:

1. Asks the configured `Expander` for query variants.
2. Asks the configured `Retriever`s to fetch candidates concurrently.
3. Asks the configured `Fuser` to merge the results (e.g., using Reciprocal Rank Fusion).
4. Asks the configured `Reranker` to finalize the list.

### 3. The Adapters (`src/search/adapters/`)

Adapters are the concrete implementations of the domain ports. They integrate with external algorithms or libraries.

- `Bm25Retriever`: Uses the in-memory term-frequency index to score documents.
- `PhraseRetriever`: Performs exact string matching.
- `SegmentVectorRetriever`: Uses the `candle` machine learning framework to embed queries and document segments, scoring them via cosine similarity.
- `RrfFuser`: Implements Reciprocal Rank Fusion to balance scores between completely different retrieval methods.
- `SynonymExpander`: A simple test-bed expander.

## Extraction Pipeline

Before search begins, files must be converted into standard `Document` objects. This is handled by `src/extract.rs` and `src/segment.rs`.

- **Format Agnostic:** PDF, HTML, Markdown, and OOXML files are all parsed into plain text.
- **Structure Aware:** Documents are broken into `Segment`s (e.g., "Page 1", "Section 2", "Slide 3") to allow localized vector embeddings and precise snippet extraction, improving semantic relevance compared to whole-document embeddings.

## Future: The Incremental Cache

Currently, `sift` is strictly transient—extracting and embedding files on every run. As the repository grows, we intend to introduce a Zig-style incremental compilation cache (`~/.cache/sift/index`) driven by content hashing (blake3), file mtimes, and inodes. This will preserve the "stateless CLI" UX while avoiding redundant work, acting as an asset pipeline rather than a traditional database.
