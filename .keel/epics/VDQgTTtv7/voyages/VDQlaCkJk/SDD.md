# SDD: Implement Scanned PDF Fallback

## Architecture Overview

The PDF extraction pipeline in `src/extract.rs` will be enhanced with a heuristic fallback. When `pdf_extract` is used, it often returns an empty string or just whitespace for image-only (scanned) PDFs. We will introduce a threshold check on the extracted text. If the text fails this check, and the `ocr` feature is enabled, we will attempt to extract the text using the OCR pipeline.

## Component Changes

### `src/extract.rs`
- **`extract_pdf`**: Update this function to store the result of `pdf_extract::extract_text_from_mem`.
- **Heuristic Check**: Implement `is_scanned_pdf_heuristic(text: &str) -> bool` which checks if the trimmed text is empty or contains fewer than `MIN_PDF_TEXT_LEN` (e.g., 20) alphanumeric characters.
- **Fallback Trigger**: If the heuristic returns true and the `ocr` feature is enabled, we will write the PDF bytes to a temporary file (since `tesseract-rs::process_pages` requires a file path) and run it through `extract_image_from_path` (a refactored version of `extract_image` that doesn't hardcode `SourceKind::Image`). Alternatively, we will attempt to see if Tesseract natively supports reading the PDF directly. *Note: If Tesseract fails to read the PDF directly, we will log an error and document the limitation, as adding full PDF-to-image rendering (like Ghostscript/Poppler) is out of scope for our lightweight goal.*

### Refactoring
- Rename the inner logic of `extract_image` (under the `ocr` feature) to a helper function `perform_ocr(path: &Path) -> Result<String>` so it can be reused by both `extract_image` and `extract_pdf`.

## Data Flow
1. `extract_pdf` is called with PDF bytes.
2. `pdf-extract` attempts to extract text.
3. The resulting text is evaluated by the heuristic.
4. If it's a standard PDF (lots of text), return the `ExtractedDocument`.
5. If it's a scanned PDF (little/no text):
   - Check if `#[cfg(feature = "ocr")]` is active.
   - If not, log a warning and return the empty text.
   - If active, write the bytes to a temp file, call `perform_ocr`, and return the OCR'd text with `SourceKind::Pdf`.
