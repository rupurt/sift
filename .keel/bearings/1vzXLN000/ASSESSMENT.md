# Recover Hybrid Retrieval Viability — Assessment

## Recommendation

Plan an explicit recovery voyage that introduces a persisted local embedding
sidecar for structure-aware segments, with freshness checks and benchmark
evidence against the current exact no-index path.

## Why

The benchmark evidence now changes the decision boundary:

- the no-index exact-search design is functionally validated;
- the same design is not operationally viable against the current latency
  target;
- sampled quality evidence does not show a retrieval win over BM25.

That is the threshold the original product brief set for reconsidering
persisted sidecar indexes.

## Proposed Next Planning Scope

- define the sidecar format and freshness contract;
- add an explicit local build/update command;
- switch hybrid search to use the sidecar-backed vector path;
- benchmark the recovered architecture against the current voyage's evidence.

## Decision

Lay this bearing and use it to seed the next epic/voyage for hybrid-retrieval
recovery.
