# SDD: Implement Base OCR Support

## Architecture Overview

The extraction pipeline in `src/extract.rs` will be extended to detect image file extensions and route them to a new OCR-based extraction function. This will use the `tesseract-rs` crate as the bridge to the Tesseract engine.

## Component Changes

### 1. `src/extract.rs`
- Add new `SourceKind` variants for `Png`, `Jpeg`, `Tiff`, `Bmp`.
- Update `extract_path` to include an `is_image_path` check.
- Implement `extract_image` using `tesseract-rs`.
- Wrap OCR-specific code in `#[cfg(feature = "ocr")]` blocks.

### 2. `src/segment.rs`
- Update `build_segments` and `default_label` to handle the new image `SourceKind` variants.
- For images, the default label should be "image 1".

### 3. `Cargo.toml`
- Add optional dependencies: `tesseract-rs` and `image`.
- Define a new `ocr` feature that enables these dependencies.

## Data Flow
1. `load_search_corpus` finds an image file (e.g., `screenshot.png`).
2. `extract_path` identifies it as an image based on the extension.
3. If `ocr` feature is enabled:
   a. The file is loaded via `image`.
   b. The image is passed to `Tesseract::ocr_from_image`.
   c. The resulting text is returned as an `ExtractedDocument`.
4. The text is segmented and indexed by the standard `sift` pipeline.

## Verification Design
- **Unit Tests**: Mock or provide a small sample image with known text for `extract_image` verification.
- **Integration Tests**: Verify that `just search` finds results within an image file when the `ocr` feature is on.
