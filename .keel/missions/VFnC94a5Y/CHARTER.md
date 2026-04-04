# Build Sector-Aware Frontier Search Cache Reuse - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Replace whole-corpus restart rescans with a sector-aware cache validity model so unchanged sectors can be reused immediately across fresh `sift` processes. | board: VFnCKDDhj |
| MG-02 | Preserve frontier hunting by allowing useful search over partial sector coverage without requiring a prior fully sealed global index. | board: VFnCKDDhj |

## Constraints

- Preserve Sift's local-first single-binary contract; the first architecture slice must not require a daemon or external database.
- Keep partial-search semantics explicit so callers can distinguish frontier, converging, and sealed coverage states.
- Treat sector hashes as cache-validity proofs and breadcrumbs as resumable indexing state; do not collapse the two concepts into one opaque cache artifact.

## Halting Rules

- DO NOT halt while epic `VFnCKDDhj` still has defining, planning, or execution work needed to remove restart-time whole-corpus rescans.
- HALT when epic `VFnCKDDhj` is achieved and Sift has a planned path for sector-aware cache validation, resumable breadcrumbs, and frontier-search semantics that do not wait on a prior sealed global index.
- YIELD to human if useful frontier scoring appears to require relaxing the single-binary local-first contract or materially changing the user-facing meaning of search completeness.
