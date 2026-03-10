# Charter: Support OCR and Additional Binary Document Formats

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Support text extraction from common image formats (PNG, JPEG, TIFF, BMP) via OCR | board: VDQgTTtv7 |
| MG-02 | Implement a fallback mechanism for scanned PDFs (image-only) using OCR | board: VDQgTTtv7 |
| MG-03 | Improve binary document extraction for existing formats (Office, PDF) where applicable | board: VDQgTTtv7 |
| MG-04 | Evaluate and minimize the performance impact of OCR on indexing latency | metric: p50_ms |
| MG-05 | Maintain a clean, platform-compatible dependency model for OCR | board: VDQgTTtv7 |

## Constraints

- **Minimal Dependencies**: Prefer high-quality, lightweight Rust libraries or well-established C-system dependencies (like Tesseract) if necessary.
- **Indexless Architecture**: The extraction must happen during the search/load process as per `sift`'s "indexless" design.
- **Performance**: OCR is expensive; it should be optional or only triggered when necessary (e.g., no text in PDF).

## Halting Rules

- Mission achieved when OCR and binary format support is integrated, verified with tests, and documented.
- Mission aborted if the required dependencies or performance overhead violate `sift`'s core principles.
