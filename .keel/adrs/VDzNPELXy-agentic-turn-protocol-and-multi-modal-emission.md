---
id: VDzNPELXy
index: 2
title: Agentic Turn Protocol and Multi-Modal Emission
status: accepted
decided_at: 2026-03-15T18:28:00
---

# Agentic Turn Protocol and Multi-Modal Emission

## Status

**Accepted** — Strategic direction for the next major evolution of the retrieval system.

## Context

Current search results are optimized for CLI presentation of local files. A new use case has emerged: searching and surfacing "turns" from coding agents (like Gemini CLI). This requires a shift from "Document-as-File" to "Document-as-Turn" and demands flexible emission options—sometimes for a conversational shell with rich highlighting, and other times as raw latent embeddings for external ranking systems.

## Decision

We will extend the `SearchEngine` capabilities to support **Agentic IR** through the following patterns:

1.  **The Turn Model**: Introduce a first-class `AgentTurn` record type that captures conversational metadata (role, session_id, tool_calls).
2.  **Multi-Modal Emission**: Decouple retrieval from presentation. The engine must support multiple emission modes:
    - **`emit_latent`**: Returns raw embedding vectors (Tensors) for handoff to external systems.
    - **`emit_turns`**: Returns high-level domain records for conversational interfaces.
    - **`emit_view`**: Returns rendered, highlighted text for standard CLI display.
3.  **Conversational Protocol**: Treat the interaction as a known protocol (e.g., Turn-based). This allows specialized `SearchExecution` adapters to act as "Protocol Interpreters."

## Constraints

- **MUST:** Maintain backward compatibility with the existing file-based search.
- **MUST:** Ensure `emit_latent` is performance-optimized to allow `sift` to act as a feature extractor.
- **SHOULD:** Use a context-aware highlighting engine (e.g., `syntect`) for the conversational shell emission.

## Consequences

### Positive

- **Interoperability:** Sift can now serve as a high-performance pre-processor for other AI tools.
- **Dynamic Context:** The Turn-based model allows for searching logic lineage rather than just static text.
- **Unified UX:** The same engine can power a terminal UI, a web-based chat log, and a headless ranking API.

### Negative

- **Model Divergence:** Balancing the needs of "Files" and "Turns" in a single `SearchStorage` trait may require more generic design.

## Verification

| Check | Type | Description |
|-------|------|-------------|
| Turn Retrieval | manual | Verify a mock Turn-based storage returns correct conversational records. |
| Embedding Emission | automated | Verify `emit_latent` returns vectors of the expected dimensionality. |

## References

- `RESEARCH.md`
- [Model Context Protocol (MCP)](https://modelcontextprotocol.io/)
