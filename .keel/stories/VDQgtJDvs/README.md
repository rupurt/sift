---
id: VDQgtJDvs
title: Support OCR For Common Image Formats
type: feat
status: icebox
scope: VDQgTTtv7/VDQgokSvk
---

# Story: Support OCR For Common Image Formats

## Acceptance Criteria
- [ ] Implement `extract_image` in `src/extract.rs` using `tesseract-rs`. [SRS-01/AC-01]
- [ ] Add `Png`, `Jpeg`, `Tiff`, `Bmp` to `SourceKind` and update `extract_path` to detect them. [SRS-02/AC-02]
- [ ] Wrap OCR dependencies and implementation in an `ocr` feature flag. [SRS-03/AC-03]
- [ ] Update `build_segments` in `src/segment.rs` to handle image formats. [SRS-04/AC-04]
- [ ] Provide unit tests in `src/extract.rs` verifying OCR extraction from a sample image. [SRS-04/AC-05]

## Evidence
- **test**: Run `cargo test --features ocr` and verify OCR extraction.
- **cmd**: Run `just search --features ocr tests/fixtures/ocr-test.png "text in image"`.
- **manual**: Verify that the binary build works without the `ocr` feature.
