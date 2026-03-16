---
id: VE1HqhVsi
index: 3
title: Library Engine and Reactor API
status: proposed
decided_at: 2026-03-16T18:00:00
---

# Library Engine and Reactor API

## Status

**Proposed** — Formalizing the transition from a search-centric facade to a physics-aligned reactor API.

## Context

The current library facade (`Sift::search`) treats retrieval as a linear, high-level operation. This is sufficient for the CLI but limits library consumers who need to control the "physics" of the system—specifically the configuration of the magnetic field (Retrieval) and the type of energy released through the emission ports (Fusion results).

## Decision

We will refactor the library API to be an **Engine/Reactor API**. The `Sift` struct will evolve into (or orchestrate) a `Reactor`.

1.  **Reactor Control**: The API will allow consumers to define a `SearchGraph` (the reactor core configuration) rather than just choosing a preset strategy.
2.  **Emission Ports**: The `execute` method will support multi-modal emission:
    - `Emission::Latent`: Raw vectors/tensors.
    - `Emission::Protocol`: Structured domain records (e.g., Agent Turns).
    - `Emission::Visual`: Rendered views.
3.  **CLI Preservation**: The `sift` executable will remain a "Search API" wrapper around the Reactor, maintaining its simple UX for end-users.

## Constraints

- **MUST:** Decouple the high-level `SearchResponse` from the core execution result.
- **MUST:** Provide a standard "Search Reactor" configuration that matches current behavior for easy migration.
- **SHOULD:** Use generics or enums to handle the different emission types at the type level where possible.

## Consequences

### Positive

- **Strategic Flexibility**: Consumers can build custom reactors for specialized domains (like agent logs).
- **Interoperability**: Headless embedding emission allows Sift to be used as a pre-processor in larger AI pipelines.
- **World Model Alignment**: The code now matches our conceptual model of Magnetism and Fusion.

### Negative

- **API Breaking Change**: Existing library consumers will need to migrate to the new Reactor-based pattern.

## Verification

| Check | Type | Description |
|-------|------|-------------|
| Multi-Modal Search | manual | Verify that the same reactor can emit both latent vectors and rendered turns. |
| CLI Compatibility | automated | Ensure the `sift` binary still produces identical output for all strategies. |

## References

- `WORLD.md`
- `RESEARCH.md`
- `ARCHITECTURE.md`
