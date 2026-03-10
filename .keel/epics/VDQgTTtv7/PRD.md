# OCR and Binary Format Support Options - Product Requirements

> We can extend `sift`'s reach into images and scanned documents using a modern OCR library (e.g., Tesseract) or a Rust-native vision model (via `candle`).

## Problem Statement

Currently, `sift` skips images and fails to extract text from scanned (image-only) PDFs. Users with local document archives often have these non-text formats, making them invisible to the current retrieval engine.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Validate bearing recommendation in delivery flow | Adoption signal | Initial rollout complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Product/Delivery Owner | Coordinates planning and execution | Reliable strategic direction |

## Scope

### In Scope
- [SCOPE-01] Support for image files (PNG, JPEG, etc.) via OCR.
- [SCOPE-02] Integration with Tesseract (via `tesseract-rs`).
- [SCOPE-03] Updated `SourceKind` and `extract_path` logic.
- [SCOPE-04] Basic unit tests for image extraction.

### Out of Scope
- [SCOPE-05] Scanned PDF fallback (deferred to later voyage).
- [SCOPE-06] Advanced image preprocessing (deskew, denoising).
- [SCOPE-07] Unrelated platform-wide refactors outside bearing findings.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Implement the core user workflow identified in bearing research. | GOAL-01 | must | Converts research recommendation into executable product capability. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Ensure deterministic behavior and operational visibility for the delivered workflow. | GOAL-01 | must | Keeps delivery safe and auditable during rollout. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove functional behavior through story-level verification evidence mapped to voyage requirements.
- Validate non-functional posture with operational checks and documented artifacts.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Bearing findings reflect current user needs | Scope may need re-planning | Re-check feedback during first voyage |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which rollout constraints should gate broader adoption? | Product | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Identification of a high-quality, platform-compatible OCR library or model. [manual:research]
- [ ] Strategy for falling back to OCR when PDF text extraction returns empty. [board:src/extract.rs]
- [ ] Clear performance/latency trade-offs documented for OCR-enabled searches. [manual:survey]
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

### Findings

- OCR is feasible via `tesseract-rs` for high accuracy or `ocrs` for pure Rust portability [SRC-01] [SRC-02]
- Scanned PDFs can be detected using empty text layer heuristics and `lopdf` [SRC-03]

### Dependencies

- `tesseract-rs` for the OCR engine [SRC-01]
- `image` for image loading and processing [SRC-01]
- `lopdf` for enhanced PDF structure detection [SRC-03]

### Alternatives Considered

- Use `tesseract-rs` for mature, system-linked OCR. Accepted for initial robust support. [SRC-01]
- Use `ocrs` for a pure Rust, ONNX-based OCR solution. Deferred as a possible future fallback for standalone distribution. [SRC-02]

---

*This PRD was seeded from bearing `VDQgTTtv7`. See `bearings/VDQgTTtv7/` for original research.*
