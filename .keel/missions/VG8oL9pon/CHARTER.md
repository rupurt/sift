# Structural Retrieval Documentation Fidelity - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Raise foundational-document fidelity so the shipped structural retrieval stack is described accurately across the repo's core reference set. | board: VG8oL9Xok |
| MG-02 | Clarify strategy selection and downstream direct-search adoption for embedders such as `paddles` without changing the planner boundary. | board: VG8oL9Xok |

## Constraints

- Preserve the current runtime and public crate-root surface; this mission is documentation-first.
- Keep the `paddles` integration guidance aligned with the existing direct-search boundary rather than proposing planner changes.
- Land the documentation updates and board artifacts in the same sealing commit sequence.

## Halting Rules

- DO NOT halt while epic `VG8oL9Xok` still has unfinished board work.
- HALT when epic `VG8oL9Xok` is delivered and the foundational docs consistently describe the structural retrieval substrate and downstream adoption seam.
- YIELD to human only if documenting the current behavior would require changing the direct-search versus planner ownership boundary.
