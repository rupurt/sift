# AGENTS.md

Shared guidance for AI agents working with this repository.

## Operational Lifecycle

1. **Bootstrap**: `nix develop` followed by `keel init` (if new) and `keel doctor --scene`.
2. **Upgrade**: `nix flake update keel` then `keel doctor --scene`.
3. **Execution**: Follow the **[INSTRUCTIONS.md](INSTRUCTIONS.md)** for Mission, Planning, and Delivery loops.
4. **Finalize**: `keel doctor --scene` must show a healthy heartbeat before any final synthesis.

## Foundational Documents

These define the binding constraints and workflow:

- `INSTRUCTIONS.md` — canonical workflow and operational guidance.
- `CONSTITUTION.md` — repository operating principles and binding delivery rules.
- `ARCHITECTURE.md` — current system architecture and component boundaries.
- `CONFIGURATION.md` — runtime, build, and environment configuration guidance.
- `EVALUATION.md` — evaluation datasets and methodology.
- `RESEARCH.md` — strategic architecture research and vision.
- `README.md` — repository intent and product positioning.
- `flake.nix` — Nix development environment and shared tooling.

Use this order when interpreting constraints: ADRs, then `INSTRUCTIONS.md`, then `CONSTITUTION.md`.

## Project Overview

This repository is `sift` — a standalone **Hybrid Information Retrieval (IR) system** for fast
document retrieval in agentic coding workflows.

Sift is intended to stay lightweight: a single Rust binary with no external
database requirement, combining BM25 keyword ranking with vector-backed semantic
search and LLM-based re-ranking to capture user **intent**.

### Core Commands

| Path | Purpose |
|------|---------|
| `nix develop` | repository shell and shared tooling |
| `keel ...` | planning, execution, and verification |
| `just search` | run a local search (e.g., `just search . "query"`) |
| `just test` | run the full verification suite (fmt, clippy, nextest) |
| `just eval all` | run comparative benchmarks |

### Board Directory (`.keel/`)

| Path | Contents |
|------|----------|
| `.keel/adrs/` | architecture decision records |
| `.keel/epics/` | strategic planning and PRDs |
| `.keel/missions/` | long-running objectives and charters |
| `.keel/stories/` | implementable units and evidence |
