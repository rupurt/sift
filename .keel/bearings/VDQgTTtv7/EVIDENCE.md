# OCR and Binary Format Support Options — Evidence

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | web | manual:rust-ecosystem-survey | https://crates.io/crates/tesseract-rs | 2026-03-09 | 2026-03-09 | high | high | Mature OCR library for Rust (via C++ link). |
| SRC-02 | web | manual:rust-ecosystem-survey | https://crates.io/crates/ocrs | 2026-03-09 | 2026-03-09 | medium | high | Pure Rust OCR engine using ONNX. |
| SRC-03 | web | manual:rust-ecosystem-survey | https://crates.io/crates/lopdf | 2026-03-09 | 2026-03-09 | high | high | Robust PDF parsing and structure detection. |

## Technical Research

### Feasibility
OCR is feasible using `tesseract-rs`, which is the industry standard for open-source OCR and has well-maintained Rust bindings. Integrating images and scanned PDFs into `sift`'s extraction pipeline is achievable with minimal architectural changes.

## Key Findings

1. `tesseract-rs` provides high accuracy but requires system-level C++ libraries [SRC-01].
2. `ocrs` is a lighter, pure-Rust alternative but still in earlier stages of maturity [SRC-02].
3. `lopdf` can detect empty text layers in PDFs, triggering an OCR fallback for scanned pages [SRC-03].

## Unknowns

- The exact performance overhead of OCR on various hardware (e.g., M1 Mac vs Linux server).
- Whether `ocrs` accuracy is sufficient for `sift`'s retrieval goals.
