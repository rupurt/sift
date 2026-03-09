---
id: 1vzJVa000
---

# Raw Document Retrieval Architecture Research — Survey

## Market Research

### Existing Solutions

The market splits into three retrieval shapes:

- lexical CLIs such as `ripgrep`, which are fast and stateless but weak on
  semantic recall
- library-driven search engines such as Tantivy, which assume an index build
  step and persistent on-disk structures
- embedding services and vector databases, which deliver semantic quality but
  violate this repository's single-binary and no-database constraints

The gap `sift` is targeting is a local CLI that behaves like a developer tool,
not a service: fast startup, no background state, and results that are good
enough for both keyword-heavy and semantically phrased agent queries.

### Competitive Landscape

Most competitive systems optimize either for indexed IR or service-style vector
search. That leaves room for an indexless or transient-index hybrid CLI, but it
also means `sift` cannot rely on the usual trick of hiding latency behind a
background ingestion service.

### Market Size

The immediate opportunity is practical rather than TAM-driven: agentic coding
workflows, local documentation search, and repository-side retrieval where
operators do not want to provision a separate database or keep sidecar state in
sync with the filesystem.

## Technical Research

### Feasibility

The technical challenge is not lexical search. The `bm25` crate already
provides a light-weight in-memory BM25 search engine, plus lower-level embedder
and scorer APIs for cases where we manage raw documents ourselves:

- https://docs.rs/bm25/latest/bm25/

Dense retrieval is the harder part. Pure-Rust inference is feasible with
Candle, whose README explicitly positions it for lightweight, serverless
deployment and Python-free production use. Candle also ships examples for BERT
and JinaBert sentence-embedding style models:

- https://github.com/huggingface/candle

Burn remains a credible future path rather than the fastest MVP path. The
`burn-candle` backend supports Linux and macOS targets and is documented as
usable for inference, while `burn-import` can turn ONNX models into Rust source
plus model weights:

- https://docs.rs/crate/burn-candle/latest
- https://docs.rs/burn-import/latest/burn_import/onnx/struct.ModelGen.html

For the initial model family, `sentence-transformers/all-MiniLM-L6-v2` is a
good fit for the MVP because it is small and standard:

- 384-dimensional embeddings
- intended for semantic search and short paragraph encoding
- truncates beyond 256 word pieces
- model size reported at 22.7M parameters

Source:

- https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2

This strongly suggests a CPU-first Rust implementation is practical, but only
if semantic scoring is bounded to a shortlist instead of brute-forcing the full
corpus every query.

### Prior Art

Relevant primary-source prior art:

- `bm25` crate documentation shows an in-memory search engine and raw scorer
  surface suitable for transient indexing.
- Candle documents BERT and JinaBert support for sentence-embedding workloads.
- Burn documents a Candle backend and ONNX-to-Rust import flow, which gives us
  a fallback path if direct Candle integration becomes too operationally noisy.
- Hugging Face model cards provide both the embedding contract and the reference
  pooling approach for MiniLM-family sentence transformers.
- The BEIR SciFact dataset card provides a standard corpus/queries/qrels layout
  that can be materialized directly into local text files.

### Proof of Concepts

Research-stage evidence collected locally:

- `uname -a`
  Linux x86_64, kernel `6.17.0-14-generic`
- `lscpu`
  AMD Ryzen Threadripper 3960X, 24 cores / 48 threads
- `free -h`
  121 GiB RAM

These numbers describe the current development workstation, not the acceptance
target. The benchmark contract therefore needs two views:

- exact local hardware evidence for reproducibility
- bounded-thread runs that better approximate laptop-class execution

## User Research

### Target Users

The target users are developers and coding agents searching local repositories,
docs directories, notes, and exported artifacts without wanting to maintain a
secondary search service.

### Pain Points

Their current pain points are consistent:

- keyword tools miss semantically related passages
- vector databases impose indexing, invalidation, and operational drag
- many local embedding solutions assume Python, libtorch, or a resident server
- persisted indexes age badly when the underlying filesystem changes often

### Validation

The operating contract itself is the strongest validation artifact here. It
explicitly prioritizes developer-workflow ergonomics over service-style search
architecture and requires raw-document search research before implementation.

## Architecture Comparison

| Approach | Constraint Fit | Latency Outlook | Complexity | Assessment |
|----------|----------------|-----------------|------------|------------|
| JIT raw-file scan over entire corpus for both BM25 and dense scoring | Excellent | BM25 viable, full-corpus dense scoring unlikely to meet 200 ms | Low to medium | Too slow for default dense ranking across thousands of files |
| Transient per-query indexing with BM25 over all files and dense rerank on a shortlist | Excellent | Best chance to stay below target with bounded semantic work | Medium | Recommended MVP architecture |
| Persisted sidecar index for sparse and/or dense structures | Poor under current contract | Best steady-state latency | High operationally | Defer unless benchmarks prove transient approaches insufficient |

## Recommended Architecture

The recommended MVP shape is a bounded hybrid pipeline:

1. Walk the filesystem for supported files at query time.
2. Decode ASCII and UTF-8 text directly from disk.
3. Build transient BM25-ready document structures in memory.
4. Retrieve a lexical shortlist across the full corpus.
5. Encode the query once with a local MiniLM-family model.
6. Encode only the top-N lexical candidates for dense reranking.
7. Combine lexical and dense scores with a simple reciprocal-rank or weighted
   fusion function.
8. Return snippets and metadata without persisting any sidecar index.

This is still hybrid-by-default because both sparse and dense signals contribute
to final ranking. The crucial optimization is that dense inference happens on a
shortlist instead of every file in the corpus.

## Evaluation Corpus

Adopt Hugging Face's BEIR SciFact dataset for the first benchmark cycle:

- dataset card: https://huggingface.co/datasets/BeIR/scifact
- corpus viewer reports `5.18k` rows
- dataset viewer reports `queries (1.11k rows)`
- BEIR split table reports SciFact `test` with `300` queries and `5K` corpus

Why SciFact first:

- it is already structured as `corpus`, `queries`, and `qrels`
- the corpus size is large enough to stress a raw-file MVP without becoming
  prohibitively slow
- it is small enough to materialize as one local `.txt` file per document
- it gives us standard IR metrics instead of anecdotal query screenshots

The materialization format should be:

- one UTF-8 `.txt` file per corpus document
- filename derived from document `_id`
- file body as `title`, blank line, then `text`
- a manifest that maps `_id` to path and retains original qrels/query IDs

## Benchmark Method

### Corpus Shape

- Primary quality corpus: BEIR SciFact `test`
- Primary performance corpus: the same materialized local text corpus
- Secondary performance corpus after MVP: a synthetic filesystem-shaped corpus
  with directory fanout and mixed file lengths to mirror repo usage more
  closely

### Quality Methodology

Measure and compare:

- `NDCG@10`
- `MRR@10`
- `Recall@10`

Evaluation rule:

- BM25-only is the baseline
- hybrid must beat or clearly explain any shortfall against BM25-only on the
  SciFact test qrels

### Latency Methodology

Measure end-to-end CLI latency separately for:

- cold file-cache runs
- warm file-cache runs
- single-query interactive usage
- batch evaluation usage

Each benchmark record must capture:

- exact command line
- git SHA
- corpus size in files and bytes
- thread count
- hardware summary
- p50, p90, and worst-case latency

Planned benchmark commands for implementation stories:

```bash
cargo run --release -- eval download scifact --out .cache/eval/scifact
cargo run --release -- eval materialize scifact --source .cache/eval/scifact --out .cache/eval/scifact-files
cargo run --release -- bench latency --corpus .cache/eval/scifact-files --queries .cache/eval/scifact/test-queries.tsv --engine bm25
cargo run --release -- bench latency --corpus .cache/eval/scifact-files --queries .cache/eval/scifact/test-queries.tsv --engine hybrid
cargo run --release -- bench quality --corpus .cache/eval/scifact-files --qrels .cache/eval/scifact/qrels/test.tsv --engine bm25
cargo run --release -- bench quality --corpus .cache/eval/scifact-files --qrels .cache/eval/scifact/qrels/test.tsv --engine hybrid
```

### Hardware Assumptions

Report the local machine exactly, but normalize acceptance runs by limiting
parallelism so workstation-class hardware does not hide poor single-query
behavior. For the current host, record:

- OS: Linux x86_64
- CPU: AMD Ryzen Threadripper 3960X
- RAM: 121 GiB

The acceptance target remains "laptop-class machine," so later implementation
stories must either rerun on a smaller host or justify bounded-thread results as
an approximation.

## Key Findings

1. Persisted sidecar indexes are not required for the MVP architecture and
   should stay deferred until transient approaches are benchmarked.
2. The likely winning shape is global BM25 plus dense reranking of a bounded
   shortlist, not full-corpus dense scoring.
3. Candle is the most direct pure-Rust inference path for the MVP; Burn remains
   a strong fallback or later optimization path via `burn-candle` and
   `burn-import`.
4. BEIR SciFact is a suitable first evaluation corpus because it already gives
   us raw corpus text, queries, and qrels at a manageable size.

## Unknowns

- Whether direct Candle CPU inference can consistently satisfy the 200 ms goal
  once model load, tokenization, and dense reranking are all included.
- How much chunking is necessary for long files before semantic quality starts
  improving enough to justify the added latency.
- Whether a second corpus closer to developer documentation will reveal
  different failure modes than SciFact.

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | web | manual:web-search | https://docs.rs/bm25/latest/bm25/ | 2026-03-08 | 2026-03-08 | high | high | bm25 crate provides in-memory BM25 search engine and raw scorer APIs |
| SRC-02 | web | manual:github-review | https://github.com/huggingface/candle | 2026-03-08 | 2026-03-08 | high | high | Candle positions itself for lightweight serverless deployment with BERT and JinaBert examples |
| SRC-03 | web | manual:web-search | https://docs.rs/crate/burn-candle/latest | 2026-03-08 | 2026-03-08 | medium | high | burn-candle backend supports Linux and macOS inference targets |
| SRC-04 | web | manual:web-search | https://docs.rs/burn-import/latest/burn_import/onnx/struct.ModelGen.html | 2026-03-08 | 2026-03-08 | medium | high | burn-import can convert ONNX models into Rust source plus model weights |
| SRC-05 | web | manual:web-search | https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2 | 2026-03-08 | 2026-03-08 | high | high | all-MiniLM-L6-v2 model card documents 384-dim embeddings and 22.7M parameters |
| SRC-06 | web | manual:web-search | https://huggingface.co/datasets/BeIR/scifact | 2026-03-08 | 2026-03-08 | high | high | BEIR SciFact dataset provides standard corpus/queries/qrels for IR evaluation |
