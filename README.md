# sift

`sift` is a standalone Rust CLI for local document retrieval in agentic
workflows. It searches raw local corpora without a persisted index, defaults to
hybrid BM25 plus dense reranking, and keeps evaluation and benchmark workflows
inside the same binary.

## Current Contract

- Single Rust binary. No external database, daemon, or long-running service.
- Default `search` mode is hybrid BM25 plus dense reranking.
- Corpus loading is transient and query-time. Sift does not write persisted
  sidecar indexes.
- Dense inference runs locally through Candle with
  `sentence-transformers/all-MiniLM-L6-v2` as the current default model.
- Supported inputs today: ASCII and UTF-8 text, HTML, text-bearing PDF, and
  OOXML Office files (`.docx`, `.xlsx`, `.pptx`).
- Target platforms are Linux and macOS. Windows is still unverified.

## Installation

For development, enter the shared shell first:

```bash
nix develop
```

Build a release binary:

```bash
cargo build --release
./target/release/sift --help
```

Install locally from source if you want `sift` on your `PATH`:

```bash
cargo install --path .
```

## Search

Hybrid search is the default:

```bash
sift search tests/fixtures/rich-docs "architecture decision"
```

If you omit the path, `sift` searches the current directory:

```bash
sift search "architecture decision"
```

Request JSON output for agent consumption:

```bash
sift search --json tests/fixtures/rich-docs "quarterly roadmap"
```

Force lexical-only BM25 search when you want a baseline:

```bash
sift search --engine bm25 tests/fixtures/rich-docs "service catalog"
```

Override dense model settings explicitly:

```bash
sift search \
  --model-id sentence-transformers/all-MiniLM-L6-v2 \
  --max-length 40 \
  .cache/eval/scifact-files \
  "retrieval architecture"
```

## Evaluation And Benchmarks

Download and materialize the SciFact evaluation corpus:

```bash
sift eval download scifact --out .cache/eval/scifact
sift eval materialize scifact \
  --source .cache/eval/scifact \
  --out .cache/eval/scifact-files
```

Measure hybrid quality against BM25:

```bash
sift bench quality \
  --engine hybrid \
  --baseline bm25 \
  --corpus .cache/eval/scifact-files \
  --qrels .cache/eval/scifact/qrels/test.tsv
```

Measure search latency:

```bash
sift bench latency \
  --engine hybrid \
  --corpus .cache/eval/scifact-files \
  --queries .cache/eval/scifact-files/test-queries.tsv
```

## Recorded Evidence

The current README claims are grounded in board evidence already checked into
`.keel/`:

- On the recorded SciFact run over 5,183 documents, hybrid search improved
  nDCG@10 from 0.6647 to 0.6764 and MRR@10 from 0.6328 to 0.6466 over BM25,
  using `all-MiniLM-L6-v2` with shortlist 8 and `max_length` 40.
- On the same recorded hybrid latency run, p50 was 170.2 ms and p90 was
  180.8 ms, with the worst query at 214.7 ms.
- The rich-document fixture corpus now exercises HTML, PDF, `.docx`, `.xlsx`,
  and `.pptx` extraction through the same search path and benchmark loaders.

## Current Limits

- No OCR or scanned-image PDF recovery.
- No legacy binary Office formats (`.doc`, `.xls`, `.ppt`).
- No persisted database or background indexing service.

## License

MIT OR Apache-2.0
