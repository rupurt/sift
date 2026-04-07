# Fuzzy Structural Retrieval And Synthesis Signals - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Add path-aware fuzzy retrieval so approximate filename and path intent can recover relevant workspace artifacts through the supported direct-search pipeline. | board: VG7WSIxMB |
| MG-02 | Promote structural evidence with deterministic bonuses so filename, heading, and definition-like matches are visible to direct search and synthesis consumers. | board: VG7WSIxMB |
| MG-03 | Add typo-tolerant fuzzy line/segment retrieval that returns snippet-bearing evidence for downstream `paddles` synthesis and context assembly. | board: VG7WSIxMB |

## Constraints

- Preserve the local-first, single-binary, no-daemon contract.
- Keep `sift` on the direct retrieval boundary; do not add recursive planning on behalf of `paddles`.
- Prefer deterministic structural heuristics over editor-memory signals such as frecency or current-buffer penalties.
- Land code, board artifacts, and foundational documentation in the same sealing commit sequence.

## Halting Rules

- DO NOT halt while the structural retrieval epic still has unplanned or unsubmitted voyage work.
- HALT when epic `VG7WSIxMB` proves shipped path-aware fuzzy retrieval, structural reranking, and fuzzy line/segment evidence through code, docs, and verification.
- YIELD to human only if downstream `paddles` adoption would require changing the retrieval/planning boundary rather than extending the existing direct-search contract.
