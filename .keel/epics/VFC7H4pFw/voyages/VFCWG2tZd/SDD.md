# Add Agent Search CLI Surface - Software Design Description

> Let the shipped CLI trigger the same autonomous planner runtime through sift search --agent while preserving existing non-agent search behavior.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage layers the executable CLI on top of the supported autonomous
library runtime. It adds an agent-mode flag and planner-aware output while
leaving the existing non-agent search command path intact.

## Context & Boundaries

The library surface voyage promotes the supported autonomous runtime. This
voyage reuses that runtime from `src/main.rs` and keeps all agent-mode behavior
behind an explicit CLI flag.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────────┐ ┌─────────────┐       │
│  │ Search CLI  │ │ Autonomous  │       │
│  │ Flag        │→│ Runtime     │       │
│  └─────────────┘ └─────────────┘       │
└─────────────────────────────────────────┘
        ↑                  ↑
 [CLI User]        [Library Surface]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/main.rs` | executable entry point | Search CLI flag parsing and output | current trunk |
| supported autonomous library entry point | crate-root/runtime | Shared planner execution path | current trunk |
| existing search CLI surface | executable behavior | Regression baseline for non-agent search | current trunk |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| CLI layering | Reuse the supported library runtime directly | Prevents semantic drift between library and CLI autonomy |
| Activation | Keep autonomous planning behind `--agent` | Preserves the current default search UX |
| Output model | Extend current output shapes with planner metadata in agent mode | Keeps downstream tooling and debugging inspectable |

## Architecture

The search CLI parses agent-mode options, constructs an autonomous request, and
invokes the shared library runtime. Standard non-agent search continues to use
the current direct search path.

## Components

- **CLI flag parser**
  Purpose: capture agent-mode task/strategy input without changing default
  search invocation.
- **Request mapper**
  Purpose: translate CLI options into the autonomous request contract.
- **Planner-aware renderer**
  Purpose: expose planner strategy and trace information for agent-mode output.

## Interfaces

The CLI consumes the supported autonomous library entry point. It does not
define a second planner runtime or CLI-only planner contract.

## Data Flow

1. Parse `sift search --agent ...`.
2. Build an autonomous request from CLI options.
3. Invoke the shared autonomous library runtime.
4. Render agent-mode output/JSON with planner-aware metadata.
5. Fall back to current search behavior when `--agent` is absent.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Agent flag is missing required task data | CLI validation fails | Return a user-facing argument error | Keep invocation explicit |
| Agent-mode output drops planner metadata | JSON/output review fails | Treat as CLI regression | Reuse the shared autonomous response shape |
| Default search regresses | Existing search tests or manual proofs fail | Block CLI rollout | Keep the flag path additive only |
