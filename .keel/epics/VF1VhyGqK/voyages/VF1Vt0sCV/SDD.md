# Add Turn Traces and Agentic Evaluation - Software Design Description

> Verify agentic search with inspectable traces, multi-hop evaluation fixtures, and comparative quality/latency evidence.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage makes the new controller accountable. It adds trace artifacts that capture what happened across turns and an evaluation harness that can compare the controller to the current hybrid champion on representative local tasks.

## Context & Boundaries

This work depends on turn-native contracts and the controller runtime existing first. It does not invent the controller policy; it makes that policy inspectable and measurable.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │ Traces  │  │ Harness  │  │ Reports │ │
│  └─────────┘  └─────────┘  └─────────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/eval.rs` | internal module | Existing evaluation and comparative reporting infrastructure | current trunk |
| Turn trace contracts | internal contract | Inspectable per-turn artifacts | previous epic output |
| Controller runtime | internal runtime | Produces agentic search behavior to evaluate | previous epic output |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Trace format | Emit explicit per-turn records rather than only log lines | Logs are insufficient for replay and evaluation |
| Evaluation scope | Start with repository-local multi-hop fixtures and comparative reporting | Keeps the first harness reproducible and local |
| Comparison baseline | Use the current hybrid champion as the mandatory baseline | The mission must justify the pivot empirically |

## Architecture

The voyage extends the existing evaluation subsystem with:
- turn trace artifact capture
- agentic fixture loading and scoring
- comparative reporting against the hybrid champion

## Components

- Trace emitter: persists or serializes turn records and context actions.
- Agentic evaluation harness: runs local tasks against the controller.
- Comparative report builder: summarizes quality, latency, and trace-linked evidence relative to the baseline.

## Interfaces

- Trace artifacts should be inspectable by humans and stable enough for regression tests.
- Evaluation outputs should align with existing report shapes where practical.

## Data Flow

1. Run controller on fixture queries.
2. Capture per-turn trace artifacts.
3. Score final retrieval quality and latency.
4. Compare results against the hybrid champion.
5. Emit comparative report artifacts.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Trace artifacts are non-deterministic | Replay or diff instability | Tighten ordering and serialization rules | Re-run with deterministic fixtures |
| Agentic fixtures are too weak | Comparative signal is noisy | Expand fixture coverage | Iterate dataset design |
