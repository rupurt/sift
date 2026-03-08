# Refresh README For Current Retrieval Contract - Software Design Description

> Rewrite the repository README so it reflects the current single-binary,
> indexless hybrid search CLI, its actual commands, supported formats, and
> measured evidence.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage is a documentation cutover, not a runtime change. The design is to
rewrite the root README from authoritative local sources already present in the
repository:

- `src/main.rs` for the actual CLI surface
- completed story evidence for benchmark and rich-document facts
- the existing PRD/SRS artifacts that define current architectural constraints

The README becomes a concise public-facing synthesis of those sources.

## Context & Boundaries

The voyage touches only user-facing documentation and board artifacts needed to
plan and verify that rewrite.

```text
┌───────────────────────────────────────────────────────────┐
│                   README Contract Cutover                │
│                                                           │
│  current CLI surface  ->  README usage/examples           │
│  architecture rules   ->  README thesis/constraints       │
│  benchmark evidence   ->  README measured claims          │
└───────────────────────────────────────────────────────────┘
          ↑                         ↑
     src/main.rs              Keel evidence logs
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/main.rs` | local code | Source of truth for command names and flags | current workspace |
| `.keel/stories/1vzJfv000/EVIDENCE/` | board evidence | Hybrid quality and latency figures | current workspace |
| `.keel/stories/1vzMeL000/EVIDENCE/` | board evidence | Rich-document format and benchmark facts | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Source of truth | Use code plus recorded Keel evidence only | Prevents README drift and unverified future claims |
| Compatibility strategy | Hard cutover; remove stale claims outright | Matches repository policy and avoids dual-path docs |
| Verification style | Mix CLI help proofs with targeted text checks and manual review | Documentation accuracy depends on both executable command shape and human-readable narrative correctness |

## Architecture

The voyage has one effective component: the root README. Its sections are
re-authored to map directly to current product concerns:

- thesis and constraints
- current capabilities
- installation/developer entrypoints
- actual CLI usage
- measured evidence

## Components

| Component | Purpose | Behavior |
|-----------|---------|----------|
| `README.md` | Public product contract | Describes the current architecture and command workflows with no stale legacy language |
| Story verification evidence | Proof backing README claims | Confirms command names, format support, and cited benchmark facts |

## Interfaces

- CLI help text exposed by `cargo run -- --help`, `cargo run -- search --help`,
  `cargo run -- bench --help`, and `cargo run -- eval --help`
- Existing evidence logs consumed only as documentation inputs

## Data Flow

1. Read the current README to identify stale claims.
2. Read `src/main.rs` and existing evidence logs to collect the actual product contract.
3. Rewrite `README.md` to align with those sources.
4. Verify command examples against CLI help and verify stale claims are absent.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| README still references removed architecture or commands | text checks during verification | Fail the story until stale terms are removed | Edit the README and rerun checks |
| README examples drift from the current CLI surface | `cargo run -- ... --help` proofs fail or contradict text | Update examples to match the binary | Re-run help proofs and manual review |
| README cites unsupported metrics or features | evidence cross-check fails | Remove or restate the unsupported claim | Limit wording to facts present in code or Keel evidence |
