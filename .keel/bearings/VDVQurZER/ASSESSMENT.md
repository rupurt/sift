---
id: VDVQurZER
---

# Embeddable Library Packaging Research — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | A supported library surface would expand `sift` from a standalone tool into reusable search infrastructure for other Rust projects. |
| Confidence | 4 | The codebase already ships a library target and the CLI uses it; the remaining work is primarily API curation and packaging discipline. |
| Effort | 3 | A curated facade and boundary cleanup are meaningful but much smaller than a full architectural rewrite. |
| Risk | 2 | The main risk is committing to the wrong public API too early or over-rotating into cargo/package churn before the facade is proven. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Findings

- The repository is already structurally capable of serving as both library and
  executable because the binary composes `sift` as a crate today. [SRC-01]
  [SRC-05] [SRC-07]
- The current public API is broader than it should be because internal search
  modules and runtime concerns are re-exported directly. [SRC-02] [SRC-03]
  [SRC-04]
- A workspace split is not the first problem to solve. The first problem is
  defining one canonical embedded API instead of letting embedders reach into
  arbitrary internal modules. [SRC-02] [SRC-03]

### Opportunity Cost

The opportunity cost is spending time on packaging and API design instead of
retrieval quality, richer format support, or performance work. That trade is
worth making now because a stable library boundary will constrain how future
features are exposed, and the current repo already has the architectural
ingredients for a clean cutover. [SRC-05] [SRC-06]

### Dependencies

- A deliberate top-level facade must replace broad module re-exports. [SRC-02]
  [SRC-03]
- CLI-only concerns should stop leaking into public library types and helpers.
  [SRC-04] [SRC-05]
- The release/install story should remain anchored on the current `sift`
  package unless later evidence proves separation is necessary. [SRC-01]

### Alternatives Considered

- Keep the current public surface and simply document it.
  Rejected because it would freeze too many internals as semver commitments.
  [SRC-02] [SRC-03]
- Split into a workspace immediately.
  Deferred because the current repo already works as one package and the API
  boundary is the sharper problem. [SRC-01] [SRC-05]
- Introduce several internal crates now.
  Rejected for the next phase because it adds the most churn before the desired
  external API is even validated. [SRC-03] [SRC-04]

## Recommendation

[x] Proceed → convert to epic [SRC-01] [SRC-02] [SRC-05]
[ ] Park → revisit later [SRC-01]
[ ] Decline → document learnings [SRC-01]

Proceed with a library-first packaging epic that:

1. curates a high-level embedded API in the existing `sift` package
2. removes CLI-specific leakage from public types and modules
3. adds feature boundaries where optional heavyweight capabilities deserve them
4. defers any workspace split until the curated facade exists and can be
   evaluated on its own merits
