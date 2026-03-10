# Brief: OCR and Binary Format Support Options

## Hypothesis
We can extend `sift`'s reach into images and scanned documents using a modern OCR library (e.g., Tesseract) or a Rust-native vision model (via `candle`).

## Problem Space
Currently, `sift` skips images and fails to extract text from scanned (image-only) PDFs. Users with local document archives often have these non-text formats, making them invisible to the current retrieval engine.

## Success Criteria
- [ ] Identification of a high-quality, platform-compatible OCR library or model. [manual:research]
- [ ] Strategy for falling back to OCR when PDF text extraction returns empty. [board:src/extract.rs]
- [ ] Clear performance/latency trade-offs documented for OCR-enabled searches. [manual:survey]

## Open Questions
- Should OCR be a compile-time feature or a runtime dependency?
- Can we use a lightweight `candle`-based model for OCR to avoid C++ dependencies (like Tesseract)?
- What is the latency impact of OCR on a typical directory of images/scans?
- Are there other binary formats (e.g., RTF, older Office formats) worth supporting?
