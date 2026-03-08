# Raw Document Retrieval Architecture Research — Brief

## Hypothesis

`sift` can satisfy the no-database, no-daemon, no-persisted-index contract by
doing lexical retrieval directly over raw files at query time, then applying
semantic reranking only to a bounded shortlist. A full persisted sidecar index
is likely unnecessary for the MVP and should remain deferred unless benchmark
evidence proves otherwise.

## Problem Space

The current repository thesis and board assume a `zvec`-backed, disk-oriented
vector path. The operating contract for this repository is materially different:

- the CLI must remain a single Rust binary
- retrieval must default to hybrid BM25 plus vector ranking
- local embeddings must use a defensible pure-Rust execution path
- persisted sidecar indexes are out of scope unless research justifies them
- performance must be benchmarked and evidenced against a sub-200 ms target

The research question is therefore not "which vector database should `sift`
embed," but "what hybrid architecture can search raw local documents quickly
enough without storing a separate index on disk."

## Success Criteria

This research is valuable if it gives the delivery track a falsifiable
architecture and benchmark contract instead of a vague product thesis.

- [x] Compare JIT raw-file scanning, transient per-query indexing, and persisted
  sidecar indexes against the repository constraints.
- [x] Pick an initial pure-Rust embedding/runtime strategy and a small default
  local model family appropriate for laptop-class inference.
- [x] Select an evaluation corpus and define ranking-quality and latency
  benchmark methodology, including hardware reporting requirements.

## Open Questions

- Can CPU-only dense reranking of a BM25 shortlist stay under the 200 ms target
  on a laptop-class machine once model load and tokenization are included?
- Is direct Candle inference sufficient for the MVP, or does Burn codegen from
  ONNX become necessary to reduce runtime overhead?
- Do we need a second, code-oriented corpus after SciFact to reflect agentic
  coding workflows more directly?
