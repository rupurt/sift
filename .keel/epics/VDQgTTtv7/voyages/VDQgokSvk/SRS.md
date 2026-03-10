# SRS: Implement Base OCR Support

## Scope

<!-- BEGIN SCOPE -->
### In scope
- [SCOPE-01] Support for image files (PNG, JPEG, etc.) via OCR.
- [SCOPE-02] Integration with Tesseract (via `tesseract-rs`).
- [SCOPE-03] Updated `SourceKind` and `extract_path` logic.
- [SCOPE-04] Basic unit tests for image extraction.

### Out of scope
- [SCOPE-05] Scanned PDF fallback (deferred to later voyage).
- [SCOPE-06] Advanced image preprocessing (deskew, denoising).
<!-- END SCOPE -->

## Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Priority | Verification |
|----|-------------|--------|-------|----------|--------------|
| SRS-01 | `extract_path` must detect image extensions and route to OCR. | FR-01 | SCOPE-01, SCOPE-03 | must | manual: Inspect src/extract.rs |
| SRS-02 | The OCR engine must use Tesseract via `tesseract-rs`. | FR-01 | SCOPE-02 | must | manual: Inspect Cargo.toml |
| SRS-04 | Extracted text from images must be segmented by the existing pipeline. | FR-01 | SCOPE-01 | must | command: cargo test --features ocr |
<!-- END FUNCTIONAL_REQUIREMENTS -->

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Priority | Verification |
|----|-------------|--------|-------|----------|--------------|
| SRS-03 | Image extraction must be available under a compile-time feature `ocr`. | NFR-01 | SCOPE-02 | should | manual: Build test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Success Criteria
- [x] Images are searchable by their text content.
- [x] No performance regression for non-image files.
- [x] Clean build with and without the `ocr` feature.
