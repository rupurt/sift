# Expose Stable Model Preparation API - Software Requirements Specification

> Add a crate-root preparation seam that can reuse compatible artifacts or invoke metamorph to translate GGUF sources into the current Candle-loadable bundle contract.

**Epic:** [VFkJD2NwK](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Export stable crate-root model-preparation types and a `prepare_model`-style API for downstream embedders.
- [SCOPE-02] Support `local-path` and `hf://repo[@revision]` model sources for the current Candle-loadable safetensors bundle contract.
- [SCOPE-03] Detect and reuse already-compatible local bundles or compatible remote Hugging Face artifacts.
- [SCOPE-04] Convert representative GGUF sources through metamorph when direct runtime loading is not possible.
- [SCOPE-05] Validate prepared artifacts and document the compatibility tradeoff in the supported library docs.

### Out of Scope

- [SCOPE-06] Paddles-specific provider, UI, or workflow behavior inside sift.
- [SCOPE-07] Native 1-bit execution support, MLX execution support, or claims that GGUF conversion preserves the original runtime profile.
- [SCOPE-08] Exposing metamorph types as sift's public API or absorbing metamorph's planner/executor surfaces into sift.
- [SCOPE-09] General runtime-contract support beyond the current Candle-loadable safetensors bundle shape.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The first stable runtime contract can be modeled as a local bundle containing `config.json`, `tokenizer.json`, and `model.safetensors`, with `generation_config.json` optional metadata. | assumption | The voyage would need a broader or different contract model before it can stay honest. |
| Metamorph's current GGUF -> `hf-safetensors` execution path is usable as a library dependency hidden behind sift's public seam. | dependency | The voyage would need a fallback execution path or a narrower initial source set. |
| Existing Qwen/Gemma/Jina Candle adapters can consume validated bundle paths returned by the new seam without needing direct HF download helpers. | dependency | Internal loaders would continue duplicating preparation logic and the seam would stay incomplete. |

## Constraints

- Keep the public API at the crate root; do not require downstream callers to use `sift::internal::*` for model acquisition or preparation.
- Preserve sift's local-first, zero-daemon, library-friendly contract.
- Keep metamorph behind a sift-owned boundary; downstream callers should deal with sift types, not metamorph types.
- Document conversion honestly as a compatibility path that may be lossy and slower than native source runtimes.
- Prefer deterministic cache paths so repeated preparation requests can reuse prior compatible or converted outputs.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The crate root must export stable source, runtime-contract, prepared-model, and preparation-mode types plus a `prepare_model` entry point for downstream embedders. | SCOPE-01 | FR-01 | facade test + source inspection |
| SRS-02 | The preparation API must accept local paths and `hf://repo[@revision]` sources and return validated local bundle paths for directly compatible Candle-loadable artifacts. | SCOPE-02, SCOPE-03 | FR-02 | automated tests for compatible preparation/reuse |
| SRS-03 | When a source is GGUF-backed and not directly loadable, sift must route preparation through metamorph and return a validated local Candle-loadable bundle plus compatibility metadata. | SCOPE-02, SCOPE-04 | FR-04 | automated mocked GGUF conversion proof |
| SRS-04 | The current Qwen/Gemma/Jina local model loaders must reuse the new seam instead of owning their own direct HF preparation flow. | SCOPE-01, SCOPE-03 | FR-05 | source inspection + targeted regression tests |
| SRS-05 | Supported library documentation must explain the new preparation seam, its supported source forms, and the fact that GGUF translation is compatibility preparation rather than native 1-bit support. | SCOPE-05 | NFR-03 | docs review + grep proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Preparation outputs must use deterministic local paths and stable reuse behavior so repeat calls do not redownload or reconvert unnecessarily. | SCOPE-03, SCOPE-04 | NFR-02 | automated tests over repeated preparation calls |
| SRS-NFR-02 | The voyage must preserve sift's local-first architectural ownership by hiding metamorph behind sift's public seam and avoiding paddles- or provider-specific API tokens. | SCOPE-01, SCOPE-05 | NFR-01 | source inspection + docs review |
| SRS-NFR-03 | Documentation and metadata must mark GGUF conversion as lossy compatibility preparation rather than native execution support. | SCOPE-04, SCOPE-05 | NFR-03 | docs review + metadata assertions |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
