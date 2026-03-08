---
created_at: 2026-03-08T15:21:14
---

# Reflection - Add Candle Dense Reranking And Hybrid Fusion

## Knowledge

- [1vzMBJ000](../../knowledge/1vzMBJ000.md) Sequence Length Dominates Hybrid Reranker Latency
- [1vzMBJ001](../../knowledge/1vzMBJ001.md) Keep Keel Verify Proofs Bounded

## Observations

The story only stabilized once the dense path became explicitly configurable.
Exposing `model_id`, `revision`, and especially `max_length` through the CLI and
benchmark metadata made it possible to tune the hybrid default against actual
quality and latency evidence instead of guessing from model names.

The hardest part was balancing the default model choice against the 200 ms
contract. The small L3 model stayed fast but did not materially improve SciFact
quality, while the initial L6 settings improved quality but missed latency.
Benchmarking showed that shortening the token budget from 48 to 40 recovered
the latency target without giving up the measured relevance gain.

The other surprise was operational rather than architectural: long benchmark
commands were fine as recorded evidence but too brittle for `keel verify`.
Separating fast verifier proofs from full benchmark evidence kept the board
artifacts reliable without weakening the acceptance evidence.
