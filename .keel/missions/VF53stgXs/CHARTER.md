# Simplify Sift Around A Context Artifact Substrate - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Introduce a first-class `ContextArtifact` substrate that can represent files, project docs, agent turns, tool outputs, environment facts, and related evidence through one domain model with explicit provenance, freshness, and budget metadata. | board: VF53stqXt |
| MG-02 | Deliver explicit acquisition adapters and a shared local-first artifact pipeline so context sources are ingested through one normalization, caching, and indexing path instead of source-specific special cases. | board: VF53su1Xu |
| MG-03 | Simplify the runtime and public surface so context assembly, emissions, and controller integration operate over artifacts through a smaller inspectable API rather than parallel file, turn, and speculative graph abstractions. | board: VF53suDXv |

## Constraints

- MUST simplify the architecture: prefer a small set of context primitives (`ContextArtifact`, acquisition adapters, and context assembly) over introducing broader graph or reactor abstractions until concrete product pressure proves they are necessary.
- MUST keep the current single-turn hybrid file search path working while the substrate is introduced.
- MUST preserve the constitution-level local-first, zero-daemon, no-external-database contract.
- MUST make the first voyage strictly local-only; remote, web, or MCP-backed adapters belong to later slices after the local substrate is proven.
- MUST make provenance, freshness, and budget metadata explicit enough for ranking, pruning, tracing, and replay.
- MUST reuse the existing cache and preparation pipeline where practical rather than creating a second indexing stack for non-file context.
- MUST make optional remote or network-backed context sources adapters over the same substrate, not a separate hosted-first architecture.
- MUST make `ContextArtifact` the primary domain type and perform a hard cutover from `Document`-centric modeling rather than introducing a supported compatibility layer.
- MUST treat this mission as simplification work in support of the active hybrid-and-agentic runtime mission, not as a parallel replacement runtime.
- MUST NOT commit Sift to a public `SearchGraph` or generic Reactor API as the primary architecture until the artifact substrate and simplified context assembly contract are proven in code.

## Halting Rules

- DO NOT halt while any MG-* goal is missing an implemented code path or explicit verification evidence.
- HALT when Sift can acquire heterogeneous context into a shared artifact substrate and expose simplified artifact-based context assembly through supported local surfaces.
- YIELD to human if simplifying the architecture requires overturning accepted ADRs or relaxing constitution-level constraints.
- YIELD to human if optional remote adapters appear to require hosted state, daemons, or a second substrate that would undermine the local-first design.
