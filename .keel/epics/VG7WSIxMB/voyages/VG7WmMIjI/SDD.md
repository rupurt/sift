# Implement Fuzzy Path And Segment Retrieval - Software Design Description

> Add direct fuzzy retrieval and structural reranking that improve path-shaped recall and synthesis-ready evidence without breaking the paddles direct-search boundary.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends the direct-search substrate with two new structural fuzzy retrieval lanes and turns the existing structural reranker into a real scoring stage. The design keeps everything inside the current `SearchPlan -> Retriever -> Fusion -> Reranker` pipeline so downstream callers, including `paddles`, can adopt the new behavior through preset or explicit plan selection rather than through a new planner contract.

## Context & Boundaries

The work stays inside the shipped direct retrieval stack:

- add path-level fuzzy matching over artifact paths and filename stems
- add fuzzy matching over segment labels and line-oriented snippet candidates
- fuse those candidates with the existing lexical, phrase, and vector lanes
- rerank the shortlist with deterministic structural bonuses

Not in scope:

- planner-controlled fallback suggestion loops
- editor-state ranking features such as frecency or current-file penalties
- any change to the `paddles` ownership boundary for recursive planning

```
┌────────────────────────────────────────────────────────────┐
│                 Direct Search Substrate                    │
│                                                            │
│  query -> path-fuzzy / segment-fuzzy / existing lanes      │
│         -> RRF fusion -> structural reranking              │
│         -> ranked hits with snippet-bearing evidence       │
└────────────────────────────────────────────────────────────┘
             ↑                                   ↑
      direct CLI + library                 downstream paddles
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `src/search/domain.rs` | internal | Public plan and retriever policy surface that must grow additively | current repo |
| `src/search/adapters/mod.rs` | internal | Retriever and reranker implementations live here today | current repo |
| `src/search/application.rs` + `src/search/engine.rs` | internal | Runtime registration for retrievers and rerankers | current repo |
| `README.md`, `ARCHITECTURE.md`, `CONFIGURATION.md`, `WORLD.md`, `EVALUATIONS.md`, `RESEARCH.md` | internal | Foundational docs that must reflect the shipped structural retrieval substrate | current repo |
| `/home/alex/workspace/spoke-sh/paddles/SEARCH.md` | downstream reference | Boundary and evidence expectations for downstream synthesis | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Structural fuzzy scope | Add `path-fuzzy` and `segment-fuzzy` retrievers | These directly target filename/path intent and snippet-bearing synthesis evidence without replacing the existing substrate. |
| Matching approach | Use deterministic Rust-native fuzzy scoring with line/segment selection | This keeps the runtime local-first and predictable while still tolerating typos and scattered matches. |
| Downstream adoption seam | Keep behavior inside `SearchPlan` and preset helpers | `paddles` can opt in through explicit plans without handing planning authority back to `sift`. |
| Preset rollout | Add a focused `path-hybrid` preset and enrich page-index presets | Allows targeted path-heavy retrieval while making the richer benchmark presets structurally stronger. |

## Architecture

The voyage adds one structural retrieval module and one stronger reranking stage:

1. `RetrieverPolicy` gains `PathFuzzy` and `SegmentFuzzy`.
2. Runtime builders register concrete retrievers for both policies.
3. Rich presets use the new retrievers alongside BM25, phrase, and vector lanes.
4. `RrfFuser` continues to combine all candidate lists without a new fusion algorithm.
5. `PositionAwareReranker` recalculates final score using structural bonuses derived from path and snippet metadata.

## Components

### Path Fuzzy Retriever

Purpose: recover artifacts when the query is closer to a filename or path fragment than to extracted body text.

Behavior:
- score filename stem, filename, and full path separately
- reward exact component containment, boundary-aligned subsequences, and approximate matches
- return artifact candidates even when body-text BM25 has little or no recall

### Segment Fuzzy Retriever

Purpose: produce snippet-bearing candidates from typo-tolerant line and segment matches.

Behavior:
- evaluate segment labels and line-oriented text spans for fuzzy alignment with the query
- keep the best snippet and location per artifact
- expose evidence that is directly reusable by downstream synthesis consumers

### Position-Aware Reranker

Purpose: reorder the fused shortlist using deterministic structural evidence.

Behavior:
- apply bonuses for filename/path matches
- apply bonuses for heading or segment-label alignment
- apply bonuses for definition-like snippets that match symbol-shaped queries
- preserve stable ordering with deterministic tie-breaking

### Public Plan Surface

Purpose: make structural retrieval easy to adopt without stringly-typed downstream coupling.

Behavior:
- update named preset registry entries
- add crate-root `SearchPlan` helpers for richer structural retrieval defaults
- keep the direct-search boundary intact for `paddles`

## Interfaces

Planned interface changes:

- `RetrieverPolicy` adds `path-fuzzy` and `segment-fuzzy`
- richer `SearchPlan` helpers expose structural presets through the public library surface
- CLI retriever overrides accept the new retriever names through existing enum parsing
- ranked hits continue to use the existing snippet and snippet-location fields

## Data Flow

1. Query expansion produces one or more query variants.
2. BM25, phrase, path-fuzzy, segment-fuzzy, and vector retrievers independently score the prepared corpus.
3. Path fuzzy scoring operates over artifact paths and filename components.
4. Segment fuzzy scoring inspects segment labels and line-oriented snippet candidates, keeping the best evidence per artifact.
5. RRF fuses all candidate lists into one shortlist.
6. `PositionAwareReranker` applies structural bonuses and emits the final ranked result set.
7. Downstream consumers such as `paddles` receive the same `SearchHit` shape, now with stronger path recall and more synthesis-ready snippets.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Path fuzzy overwhelms lexical relevance | Tests show path-only artifacts outranking stronger content matches | Tune and cap path bonuses and fuzzy thresholds | Keep structural boosts additive and bounded |
| Segment fuzzy returns noisy snippets | Retriever tests expose low-signal line matches | Tighten alignment thresholds and prefer label-aware matches | Retain only the strongest snippet per artifact |
| Docs drift from shipped preset composition | Verification finds strategy docs missing new retrievers | Update foundational docs in the same slice | Keep preset changes and docs in one commit |
| Downstream synthesis assumes planning behavior changed | Boundary review against `paddles` docs shows drift | Keep changes limited to direct retrieval output quality | Preserve existing `SearchPlan`-based adoption path |
