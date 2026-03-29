# Promote Graph Search Library and CLI Surface - Software Design Description

> Expose bounded graph search through the supported library surface and
> existing `sift search --agent` CLI.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage promotes graph search from a runtime capability into supported
product surfaces. It should keep graph mode library-first and thread CLI
selection through the existing agent entry point instead of creating a second
command family.

## Context & Boundaries

The graph contract, runtime, planners, and evaluation should already exist.
This voyage only exposes and documents them through supported surfaces.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Library surface | Reuse the current autonomous surface patterns for graph mode | Keeps graph search consistent with existing embedding guidance |
| CLI surface | Extend `sift search --agent` rather than adding a second graph command | Avoids surface sprawl |
| Output model | Include graph metadata in supported outputs | Graph mode should remain inspectable to users and embedders |

## Components

- **Supported graph library entry point**
  Purpose: expose bounded graph search through the public crate-root contract.
- **CLI graph selector**
  Purpose: route `sift search --agent` into graph mode.
- **Graph response renderer**
  Purpose: present graph metadata in text or JSON output.

## Data Flow

1. Accept a library or CLI request that selects graph mode.
2. Route the request through the supported graph runtime.
3. Emit graph-aware response metadata.
4. Document the supported usage pattern.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Graph mode is requested without a supported runtime configuration | Surface cannot resolve graph execution | Return an explicit contract error | Fix configuration or request shape |
| Graph metadata is omitted from supported output | Response rendering loses graph trace visibility | Treat as a surface regression | Restore graph response fields before shipping |
