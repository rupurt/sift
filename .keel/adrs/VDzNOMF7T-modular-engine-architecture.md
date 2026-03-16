---
id: VDzNOMF7T
index: 1
title: Modular Engine Architecture
status: accepted
decided_at: 2026-03-15T18:25:00
---

# Modular Engine Architecture

## Status

**Accepted** — This architecture is already partially implemented in `src/search/engine.rs`.

## Context

Sift began as a linear search pipeline (`Expansion -> Retrieval -> Fusion -> Reranking`) optimized for local files. As we expand to new domains (e.g., searching agent turns) and deployment environments (e.g., library embedding), the hardcoded `SearchService` orchestrator has become a bottleneck. We need a way to swap out storage backends, execution runtimes, and strategy compilers without polluting the core domain logic.

## Decision

We will elevate the search process into a formal **Engine** governed by four foundational traits:

1.  **`SearchStorage`**: Abstracts the persistence and indexing layer. Decouples search from the local filesystem.
2.  **`SearchIR`**: Represents the "compiled" search plan as a Graph of Operations. Allows for dynamic graph generation and optimization.
3.  **`SearchExecution`**: Defines the runtime traversal of the IR graph (sequential, parallel, or async).
4.  **`SearchEngine`**: The top-level orchestrator that binds the other three traits into a unified `.search()` interface.

## Constraints

- **MUST:** Define all core logic in terms of these traits.
- **MUST:** Keep the `GenericEngine<IR, Exec, Storage>` as the primary implementation.
- **SHOULD:** Use the `EngineFactory` to simplify construction for standard use cases.
- **MUST NOT:** Leak storage-specific details (like file paths) into the `SearchIR` or `SearchExecution` logic.

## Consequences

### Positive

- **Pluggable Backends:** Sift can now search S3, databases, or remote APIs by implementing `SearchStorage`.
- **Flexible Execution:** We can introduce a `ParallelExecutor` using Rayon or an `AsyncExecutor` using Tokio without changing search strategies.
- **Strategy Optimization:** The IR layer can prune unused retrievers or combine expansion steps before execution.

### Negative

- **Trait Complexity:** Increases the abstraction level, making the codebase slightly harder for new contributors to navigate initially.
- **Boilerplate:** Requires more explicit generic parameter management in the engine orchestrator.

## Verification

| Check | Type | Description |
|-------|------|-------------|
| Trait Implementation | automated | Verify `GenericEngine` satisfies `SearchEngine` for standard adapters. |
| Strategy Loop | automated | Run `just eval all` to ensure all presets still work through the `EngineFactory`. |

## References

- `RESEARCH.md`
- `src/search/engine.rs`
- `ARCHITECTURE.md`
