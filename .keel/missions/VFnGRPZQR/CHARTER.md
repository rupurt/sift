# Implement Sector-Aware Frontier Search Cache Reuse - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Direct search warm restarts reuse clean sectors and rebuild only dirty sectors instead of validating lexical state at whole-corpus granularity. | board: VFnGRPtQQ |
| MG-02 | Interrupted sector rebuilds resume from persisted breadcrumbs and surface truthful frontier, converging, and sealed coverage to operators and embedders. | board: VFnGRPtQQ |
| MG-03 | Controller, autonomous, CLI, and library runtime surfaces adopt the shared sector-aware preparation path with end-to-end restart proofs. | board: VFnGRPtQQ |

## Constraints

- Preserve Sift's single-binary, local-first contract; no daemon or external database may become required.
- Extend the existing manifest/blob/BM25 cache substrate rather than introducing a parallel file-state authority.
- Deliver direct-search value first; later runtime adoption must layer on the same preparation path instead of blocking the first slice.
- Keep completeness honest: frontier and converging modes must never overstate sealed coverage.

## Halting Rules

- DO NOT halt while epic `VFnGRPtQQ` has planning or execution work needed to replace whole-corpus restart rescans.
- DO NOT halt before direct search ships sector reuse, breadcrumbs, coverage semantics, and shared runtime adoption in ordered slices.
- HALT when epic `VFnGRPtQQ` is verified and the runtime can reuse clean sectors, resume dirty-sector rebuilds, and report truthful coverage across shipped surfaces.
- YIELD to human only if truthful frontier ranking requires changing the user-facing meaning of result completeness or breaking the single-binary contract.
