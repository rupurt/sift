# Indexing Progress And Incremental Reuse - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | `sift search` and the library-owned search runtime reuse persisted indexing artifacts by default for unchanged corpora, including BM25 reuse in direct and autonomous flows. | board: VFlF7aHQw |
| MG-02 | Human operators and library callers can observe indexing progress with enough detail to distinguish cache reuse from fresh indexing work. | board: VFlF7aHQw |

## Constraints

- Keep the search runtime local-first and file-based; no daemon or external database.
- Preserve stdout result contracts, especially JSON mode.
- Prefer additive library surfaces over internal-only observability hooks.

## Halting Rules

- DO NOT halt while voyage VFlFAS7rn has unplanned or incomplete delivery slices.
- DO NOT halt before CLI search uses a real cache root and direct search has a public progress entry point.
- HALT when the epic proves incremental reuse plus visible indexing progress across direct and autonomous flows.
- YIELD to human only if progress detail requires a breaking public API change that cannot be handled additively.
