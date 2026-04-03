# Stable Local Model Preparation Seam - Product Requirements

## Problem Statement

Downstream embedders currently reach into sift internals to acquire or prepare local model artifacts. Newly published runtime-specific formats like GGUF and MLX deepen that leak because sift has no stable public preparation boundary that can reuse compatible bundles or invoke metamorph when conversion is required.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Give embedders one supported sift API for preparing local model artifacts for the current runtime contract. | Downstream code can prepare model artifacts through crate-root types and no longer needs `sift::internal::*` for acquisition/preparation. | One documented public seam exported from the crate root |
| GOAL-02 | Unlock compatibility for runtime-specific releases such as PrismML Bonsai 8B without forcing each integrator to reimplement conversion logic. | A representative GGUF source can be prepared into a validated Candle-loadable safetensors bundle through the sift seam. | One automated proof covering a GGUF-to-bundle path |
| GOAL-03 | Preserve sift's architectural ownership and local-first operating contract while using metamorph behind the scenes. | The implementation remains sift-owned, library-friendly, and explicitly documented as compatibility preparation rather than native 1-bit execution. | Docs and code keep metamorph hidden behind the sift surface |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Embedder / Integrator | A Rust application such as paddles embedding sift as a dependency. | One stable boundary for local model preparation that does not require internal sift imports. |
| Runtime Maintainer | A developer evolving sift's Candle-backed local model runtime. | One owned preparation path that can reuse compatible bundles and delegate conversion cleanly. |

## Scope

### In Scope

- [SCOPE-01] Add stable public source, runtime-contract, and prepared-artifact types exported from the crate root.
- [SCOPE-02] Support local paths and `hf://repo[@revision]` sources.
- [SCOPE-03] Detect and reuse already-compatible local bundles or compatible remote Hugging Face artifacts.
- [SCOPE-04] Invoke metamorph-backed conversion when GGUF sources must be translated into the current Candle-loadable safetensors bundle shape.
- [SCOPE-05] Validate prepared artifacts before returning them and document the compatibility tradeoff.

### Out of Scope

- [SCOPE-06] Provider- or UI-specific paddles logic inside sift.
- [SCOPE-07] Native 1-bit runtime support or claims that GGUF conversion preserves original runtime efficiency.
- [SCOPE-08] Absorbing metamorph into sift or exposing metamorph types as sift's public API.
- [SCOPE-09] General multi-runtime preparation support beyond the current Candle-loadable bundle contract.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Sift must expose a documented stable public API that prepares local model artifacts from a source description and an explicit runtime contract. | GOAL-01 | must | Embedders need one supported preparation boundary instead of internal imports. |
| FR-02 | The public preparation API must accept local paths and `hf://repo[@revision]` sources. | GOAL-01, GOAL-02 | must | The motivating downstream cases span both local and remote artifact locations. |
| FR-03 | The implementation must detect already-compatible Candle-loadable bundles and return validated local artifact paths without unnecessary conversion. | GOAL-01, GOAL-03 | must | Reuse is required to preserve local-first ergonomics and avoid needless work. |
| FR-04 | When a source is not directly compatible but Metamorph can translate it into the current bundle contract, sift must perform or reuse that conversion behind the public seam. | GOAL-02, GOAL-03 | must | Compatibility for GGUF-published models is the core motivating outcome. |
| FR-05 | Prepared-model results must include validated local artifact paths plus enough metadata for downstream runtime loading and compatibility reporting. | GOAL-01, GOAL-03 | must | Callers need usable bundle paths and honest preparation metadata. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The preparation seam must preserve sift's local-first, library-friendly, zero-daemon contract and keep metamorph behind a sift-owned boundary. | GOAL-01, GOAL-03 | must | The new API must simplify integration without inverting architectural ownership. |
| NFR-02 | The implementation must use deterministic local cache/output paths so compatible bundles and converted outputs can be reused predictably across calls. | GOAL-01, GOAL-02 | must | Reuse and reproducibility depend on stable local identities. |
| NFR-03 | Documentation must state clearly that GGUF-to-safetensors preparation is a compatibility path that sacrifices native 1-bit runtime efficiency. | GOAL-03 | must | The product claim must remain honest and avoid overstating runtime capabilities. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Public API | Facade tests and source review of crate-root exports | Story-level proof for stable types and function wiring |
| Compatible reuse | Unit/integration tests over local or mocked compatible bundles | Story-level proof for validation and reuse behavior |
| GGUF compatibility path | Automated test with a representative mocked `hf://prism-ml/Bonsai-8B-gguf@main` source routed through metamorph | Story-level proof for conversion and prepared output metadata |
| Documentation | Manual review plus grep-based checks in README/LIBRARY docs | Story-level proof for compatibility framing and usage guidance |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current Candle-backed runtime contract can be represented as a stable local safetensors bundle shape with explicit required files. | The public seam would need a broader runtime-contract model before it can stay honest. | Validate during voyage design and loader refactoring. |
| Metamorph's current GGUF -> `hf-safetensors` execution path is sufficient for the first compatibility proof. | The epic would need a narrower initial source set or a different integration strategy. | Validate with automated conversion proof. |
| Downstream integrators benefit even if the first slice only targets the current Candle-loadable bundle contract. | The public seam may be too narrow to replace internal usage. | Validate against the paddles-style preparation workflow. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Some existing loaders expect slightly different file sets than metamorph's stricter `hf-safetensors` bundle validation. | Epic owner | Open |
| A sibling Metamorph repo exists locally today, but sift still needs a release-safe dependency or invocation path. | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Sift exports one documented stable preparation seam at the crate root.
- [ ] A representative GGUF source can be prepared into a validated Candle-loadable bundle through the public seam.
- [ ] Downstream callers no longer need `sift::internal::*` for local model acquisition/preparation.
- [ ] Docs describe the feature as compatibility preparation, not native 1-bit execution support.
<!-- END SUCCESS_CRITERIA -->
