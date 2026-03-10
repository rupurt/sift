---
id: VDQgTTtv7
---

# OCR and Binary Format Support Options — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 4 | Enables search for image-heavy and scanned archives. |
| Confidence | 5 | Tesseract is a mature and well-tested solution. |
| Effort | 3 | Integration with `sift` extraction pipeline is straightforward. |
| Risk | 2 | Dependency on external C++ library may affect portability. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

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

## Recommendation

[x] Proceed → convert to epic [SRC-01] [SRC-03]
[ ] Park → revisit later
[ ] Decline → document learnings

Implement OCR support using `tesseract-rs` for images and scanned PDFs.
