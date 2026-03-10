---
created_at: 2026-03-09T20:18:53
---

# Reflection - Detect Scanned PDFs And Fallback To OCR

## Knowledge

- [VDQmKRtRl](../../knowledge/VDQmKRtRl.md) Scanned PDF Heuristic

## Observations

The fallback mechanism was straightforward to implement thanks to the `process_pages` method in `tesseract-rs`, which we discovered in the previous story. By abstracting `perform_ocr`, we were able to seamlessly reuse the logic for PDFs. The heuristic avoids unnecessary OCR latency for standard text PDFs.
