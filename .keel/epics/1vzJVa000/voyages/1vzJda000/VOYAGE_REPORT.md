# VOYAGE REPORT: Build Hybrid Text Retrieval MVP

## Voyage Metadata
- **ID:** 1vzJda000
- **Epic:** 1vzJVa000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Build SciFact Evaluation And Benchmark Harness
- **ID:** 1vzJfc000
- **Status:** done

#### Summary
Create the evaluation and benchmark foundation for the indexless MVP by adding
SciFact corpus download/materialization commands plus BM25-oriented quality and
latency benchmark commands that emit reproducible evidence.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `sift eval download scifact` and `sift eval materialize scifact` produce a local benchmark workspace containing stable document IDs, UTF-8 text files, query data, and qrels suitable for later CLI evaluation. <!-- verify: sh -lc 'cargo test eval:: && cargo run -- eval download scifact --out .cache/eval/scifact && cargo run -- eval materialize scifact --source .cache/eval/scifact --out .cache/eval/scifact-files', SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-01] `sift bench quality --engine bm25` and `sift bench latency --engine bm25` execute against the materialized corpus and emit structured benchmark output. <!-- verify: sh -lc 'cargo test bench:: && cargo run -- bench quality --engine bm25 --corpus .cache/eval/scifact-files --qrels .cache/eval/scifact/qrels/test.tsv && cargo run -- bench latency --engine bm25 --corpus .cache/eval/scifact-files --queries .cache/eval/scifact-files/test-queries.tsv', SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-01] Benchmark output records the exact command, git SHA, hardware summary, corpus counts, and measured timing or metric fields needed for reproducible evidence capture. <!-- verify: sh -lc 'cargo test bench:: && cargo run -- bench latency --engine bm25 --corpus .cache/eval/scifact-files --queries .cache/eval/scifact-files/test-queries.tsv', SRS-03:start:end, proof: ac-3.log -->

#### Implementation Insights
- **1vzLO0001: Keep materialized eval corpora self-contained**
  - Insight: Copying the benchmark query TSV into the materialized corpus directory keeps benchmark commands path-local and avoids coupling later stories to the raw download tree layout
  - Suggested Action: For future corpora, materialize documents plus the benchmark-facing query/qrels files into one self-contained directory structure
  - Applies To: `src/eval.rs`, `src/bench.rs`, future eval corpus adapters
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzJfc000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzJfc000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzJfc000/EVIDENCE/ac-3.log)

### Implement Raw File BM25 Search CLI
- **ID:** 1vzJfp000
- **Status:** done

#### Summary
Implement the first real search path for `sift`: recursive raw-file traversal,
ASCII/UTF-8 decoding, transient BM25 ranking, snippets, and both terminal and
JSON output without any persisted search index.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] `sift search <query> <path>` recursively scans supported ASCII/UTF-8 files and returns ranked BM25 results without requiring a prebuilt or persisted index. <!-- verify: sh -lc 'cargo test search::bm25 && cargo run -- search "retrieval architecture" .cache/eval/scifact-files --engine bm25', SRS-04:start:end, proof: ac-1.log -->
- [x] [SRS-05/AC-01] `sift search --json` emits machine-readable result entries containing path, rank, score, and snippet fields for agent consumption. <!-- verify: sh -lc 'cargo test cli::json_output && cargo run -- search --json "retrieval architecture" .cache/eval/scifact-files --engine bm25', SRS-05:start:end, proof: ac-2.log -->
- [x] [SRS-06/AC-01] Search execution does not create a persisted sidecar index or require a background service, and unsupported files are skipped deterministically rather than crashing the command. <!-- verify: sh -lc 'before=$(find .cache/eval/scifact-files -type f -print | LC_ALL=C sort | xargs sha256sum | sha256sum | cut -d" " -f1); cargo test fs::filtering && cargo run -- search "retrieval architecture" .cache/eval/scifact-files --engine bm25 >/tmp/sift-search-ac3.txt; after=$(find .cache/eval/scifact-files -type f -print | LC_ALL=C sort | xargs sha256sum | sha256sum | cut -d" " -f1); printf "corpus_digest_before=%s\ncorpus_digest_after=%s\n\n" "$before" "$after"; cat /tmp/sift-search-ac3.txt', SRS-06:start:end, proof: ac-3.log -->

#### Implementation Insights
- **1vzLal000: Separate benchmark IDs from recursive search IDs**
  - Insight: Recursive search needs stable full-path identities to avoid basename collisions, while benchmark corpora still need stem-based IDs to match qrels manifests
  - Suggested Action: Keep a shared in-memory document/index layer, but let each loader define its own canonical document ID policy
  - Applies To: `src/search.rs`, future hybrid reranking, benchmark corpus loaders
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzJfp000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzJfp000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzJfp000/EVIDENCE/ac-3.log)

### Add Candle Dense Reranking And Hybrid Fusion
- **ID:** 1vzJfv000
- **Status:** done

#### Summary
Add the dense half of the MVP by loading a local pure-Rust embedding model,
reranking a bounded lexical shortlist, fusing BM25 and dense signals into the
default search path, and proving the quality and latency behavior against the
evaluation corpus.

#### Acceptance Criteria
- [x] [SRS-07/AC-01] The default `sift search` path combines BM25 full-corpus retrieval with dense reranking on a bounded shortlist and produces one final hybrid ranking. <!-- verify: sh -lc 'cargo test hybrid::fusion && cargo run -- search "retrieval architecture" .cache/eval/scifact-files --limit 3', SRS-07:start:end, proof: ac-1.log -->
- [x] [SRS-08/AC-01] Dense inference runs through a local pure-Rust runtime and model-loading path rather than a remote API, daemon, or native service dependency. <!-- verify: sh -lc 'cargo test dense::model && cargo tree | rg "candle|rust_tokenizers" && cargo run -- search "retrieval architecture" .cache/eval/scifact-files --limit 3', SRS-08:start:end, proof: ac-2.log -->
- [x] [SRS-09/AC-01] `sift bench quality` compares BM25-only and hybrid runs on the SciFact qrels and records the exact metric delta between them. <!-- verify: sh -lc 'cargo test bench::quality', SRS-09:start:end, proof: ac-3.log -->
- [x] [SRS-10/AC-01] `sift bench latency --engine hybrid` records measured p50, p90, and worst-case latency against the 200 ms target and preserves any shortfall as explicit evidence instead of hiding it. <!-- verify: sh -lc 'cargo test bench::latency', SRS-10:start:end, proof: ac-4.log -->

#### Implementation Insights
- **1vzMBJ000: Sequence Length Dominates Hybrid Reranker Latency**
  - Insight: On the SciFact workload, lowering the dense encoder `max_length` had a much larger latency effect than reducing the BM25 shortlist. `all-MiniLM-L6-v2` at `max_length 40` kept a material quality gain over BM25 while bringing full-run hybrid latency back under the p50/p90 target.
  - Suggested Action: Treat sequence length as the primary performance dial before weakening shortlist depth or replacing the model, and record both quality and full-query latency before changing defaults.
  - Applies To: `src/dense.rs`, `src/main.rs`, `src/bench.rs`, hybrid search benchmarks
  - Category: architecture

- **1vzMBJ001: Keep Keel Verify Proofs Bounded**
  - Insight: `keel verify run` is better used for bounded smoke proofs and targeted tests; full release benchmarks are more reliable when recorded separately with `keel story record --cmd`, which preserves the exact command output in evidence logs without making the verifier brittle.
  - Suggested Action: Put the exact long benchmark commands in `story record` evidence, and keep README `verify:` annotations short enough to pass consistently under the verifier harness.
  - Applies To: `.keel/stories/*/README.md`, benchmark-heavy stories
  - Category: process


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzJfv000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzJfv000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzJfv000/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/1vzJfv000/EVIDENCE/ac-4.log)


