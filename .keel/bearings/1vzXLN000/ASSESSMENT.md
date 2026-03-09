---
id: 1vzXLN000
---

# Recover Hybrid Retrieval Viability — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | Hybrid retrieval is non-viable without recovery; this unblocks the core product contract. |
| Confidence | 4 | Benchmark evidence from the completed voyage clearly identifies the bottleneck. |
| Effort | 4 | Requires sidecar format design, build/update command, and benchmark re-validation. |
| Risk | 3 | Main risk is invalidation/freshness complexity for the persisted sidecar. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Opportunity Cost

Pursuing sidecar recovery delays further ranking refinements and format
expansion, but the current hybrid path is operationally non-viable so recovery
must come first. [SRC-01]

### Findings

- The true-hybrid voyage confirmed that exact no-index semantic retrieval is too slow for the target latency [SRC-01]
- Sampled quality evidence showed hybrid underperformed BM25 on NDCG@10 and MRR@10 [SRC-01]
- A persisted local sidecar index is the strongest candidate recovery direction [SRC-02]

### Dependencies

The following must hold:

- The completed voyage benchmark evidence validates the latency shortfall as architectural, not cosmetic [SRC-01]
- Prior bearing research confirms the sidecar approach was previously deferred pending evidence [SRC-02]

### Alternatives Considered

Alternatives considered:

- Keep exact search but tune model and segmentation.
  Rejected because the measured latency miss is too large for tuning alone to recover. [SRC-01]
- In-memory ANN or session index without persistence.
  Rejected because it still pays heavy preparation cost each run and is unlikely to recover enough latency. [SRC-01]
- Persisted local embedding sidecar with freshness checks.
  Recommended as the strongest path to sub-second hybrid retrieval while preserving local UX. [SRC-01] [SRC-02]

## Recommendation

[x] Proceed → convert to epic [SRC-01] [SRC-02]
[ ] Park → revisit later
[ ] Decline → document learnings

Plan an explicit recovery voyage that introduces a persisted local embedding
sidecar for structure-aware segments, with freshness checks and benchmark
evidence against the current exact no-index path.
