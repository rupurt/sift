# Expose Stable Model Preparation API - Software Design Description

> Add a crate-root preparation seam that can reuse compatible artifacts or invoke metamorph to translate GGUF sources into the current Candle-loadable bundle contract.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds a new crate-root model-preparation module that owns local
runtime preparation for Candle-backed model bundles. The public surface exposes
stable sift types such as `ModelSource`, `ModelRuntimeContract`, `PreparedModel`,
and `prepare_model(...)`. Internally, the module handles three cases:

1. reuse an already-compatible local bundle,
2. acquire a directly compatible Hugging Face safetensors bundle into sift's
   local cache, or
3. invoke metamorph to convert a GGUF source into the same runtime contract,
   then validate and return the prepared bundle.

Existing Qwen/Gemma/Jina model loaders stop downloading assets directly and
instead consume bundle paths returned by the new seam.

## Context & Boundaries

The seam belongs to sift and is intentionally narrower than metamorph's own
library contract. Downstream embedders should depend on sift's stable types and
not on metamorph or `sift::internal::*`.

```
┌────────────────────────────────────────────────────────────┐
│                        This Voyage                         │
│                                                            │
│  crate-root seam   compatible fetch/validate   metamorph   │
│       │                     │                     │         │
│       └──────────────┬──────┴──────────────┬──────┘         │
│                      ▼                     ▼                │
│            deterministic local prepared bundle cache       │
└────────────────────────────────────────────────────────────┘
              ↑                               ↑
       downstream embedders           internal Candle loaders
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `metamorph` | library dependency | Source inspection, compatibility planning, GGUF conversion, and remote GGUF acquisition when direct loading is impossible. | git dependency |
| existing `cache_dir(...)` helpers | internal module | Resolve deterministic local cache roots for compatible and converted bundles. | current trunk |
| `ureq` | library | Fetch directly compatible Hugging Face bundle files when no conversion is required. | current trunk |
| `tokenizers` + Candle safetensors loader | libraries | Validate tokenizer and weights artifacts before returning prepared bundles. | current trunk |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Public contract | Export sift-owned enums/structs at the crate root instead of re-exporting metamorph types. | Keeps the stable API narrow and preserves sift's ownership boundary. |
| Initial runtime target | Model one runtime contract: the current Candle-loadable safetensors bundle shape. | Matches the user request and keeps the first slice concrete. |
| Compatible Hugging Face path | Keep direct compatible fetching in sift instead of routing all compatible sources through metamorph. | Metamorph's current remote execution strength is GGUF conversion, while sift already knows the exact bundle files its runtime needs. |
| Conversion reuse | Write converted outputs into deterministic sift-managed cache paths keyed by source + contract. | Repeated calls should reuse prior work without caller-managed bookkeeping. |
| Loader integration | Refactor Qwen/Gemma/Jina loaders to consume `PreparedModel` paths and stop owning direct HF downloads. | Removes duplicate preparation logic and proves the seam is real. |

## Architecture

The new module is a small orchestration layer:

- Public facade types model source, artifact format, runtime contract, and
  preparation mode.
- A compatible-path helper validates local bundles or fetches required remote
  bundle files into the existing model cache.
- A conversion-path helper translates sift types into metamorph requests,
  executes or reuses conversion output, then validates the prepared bundle
  against sift's runtime contract.
- Existing Candle-backed model adapters depend only on `PreparedModel` paths.

## Components

- `src/model.rs`:
  Purpose: define the stable public preparation API and orchestrate compatible
  reuse, direct acquisition, conversion, and validation.
- `crate-root exports in src/lib.rs`:
  Purpose: make the model-preparation seam part of the supported library
  surface.
- updated Qwen/Gemma/Jina loaders:
  Purpose: consume `PreparedModel` bundle paths instead of direct HF helpers.
- tests for preparation:
  Purpose: prove local compatible reuse and mocked remote GGUF conversion for
  the PrismML Bonsai proof shape.

## Interfaces

- Public API:
  - `prepare_model(source, runtime_contract) -> anyhow::Result<PreparedModel>`
  - `ModelSource::{LocalPath, HuggingFace}`
  - `ModelRuntimeContract::CandleSafetensorsBundle`
  - `PreparedModel` with validated local artifact paths and preparation metadata
- Internal helper boundary:
  - translate compatible GGUF requests into metamorph `ConvertRequest`
  - keep all metamorph-specific types private to the module
- Documentation surface:
  - README and LIBRARY guide add the preparation seam and the compatibility caveat

## Data Flow

1. Caller constructs a `ModelSource` and selects
   `ModelRuntimeContract::CandleSafetensorsBundle`.
2. `prepare_model` inspects the source:
   - compatible local bundle: validate and return directly,
   - likely/direct compatible HF source: fetch required files into the model cache and validate,
   - GGUF or other incompatible source: ask metamorph for compatibility and convert into a deterministic prepared bundle path.
3. Validation ensures required files exist and that tokenizer/weights are loadable.
4. `PreparedModel` returns stable local paths plus metadata such as source
   format, preparation mode, and lossy-compatibility notes.
5. Internal Candle loaders reuse those paths instead of downloading their own
   assets.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Source is not compatible and no executable conversion path exists | source inspection or metamorph compatibility reports no usable path | return an `anyhow` error with compatibility context | caller chooses a different source or runtime contract |
| Direct HF bundle fetch is incomplete or corrupt | runtime-contract validation fails after fetch | clear/redownload the compatible cache once, then fail if still invalid | investigate source repo layout or switch to conversion path |
| Metamorph conversion is blocked by unsupported source format or execution path | metamorph compatibility report contains blockers or conversion fails | surface the blocker text through sift's error chain | caller supplies a supported source or waits for broader runtime support |
| Prepared bundle paths exist but no longer satisfy the runtime contract | validator fails on reuse | rebuild the prepared output from source when possible; otherwise fail | remove stale cache and retry |
