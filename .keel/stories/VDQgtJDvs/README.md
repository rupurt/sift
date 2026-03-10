---
id: VDQgtJDvs
title: Support OCR For Common Image Formats
type: feat
status: done
scope: VDQgTTtv7/VDQgokSvk
updated_at: 2026-03-09T20:12:00
started_at: 2026-03-09T20:01:47
submitted_at: 2026-03-09T20:09:06
completed_at: 2026-03-09T20:12:00
---

# Story: Support OCR For Common Image Formats

## Acceptance Criteria

- [x] [SRS-01/AC-01] Implement `extract_image` in `src/extract.rs` using `tesseract-rs` <!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-01] Add `Png`, `Jpeg`, `Tiff`, `Bmp` to `SourceKind` and update `extract_path` to detect them <!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-01] Wrap OCR dependencies and implementation in an `ocr` feature flag <!-- verify: manual, SRS-03:start:end, proof: ac-3.log -->
- [x] [SRS-04/AC-01] Update `build_segments` in `src/segment.rs` to handle image formats <!-- verify: manual, SRS-04:start:end, proof: ac-4.log -->
- [x] [SRS-04/AC-02] Provide unit tests in `src/extract.rs` verifying OCR extraction from a sample image <!-- verify: command, SRS-04:start:end, proof: ac-5.log -->

## Evidence
- **test**: Run `cargo test --features ocr` and verify OCR extraction.
- **cmd**: Run `just search --features ocr tests/fixtures/ocr-test.png "text in image"`.
- **manual**: Verify that the binary build works without the `ocr` feature.

## Rejections

### 2026-03-09

Resetting to re-record evidence with corrected requirement mapping
