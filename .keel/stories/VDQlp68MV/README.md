---
id: VDQlp68MV
title: Detect Scanned PDFs And Fallback To OCR
type: feat
status: done
scope: VDQgTTtv7/VDQlaCkJk
updated_at: 2026-03-09T20:19:40
started_at: 2026-03-09T20:17:08
submitted_at: 2026-03-09T20:19:40
completed_at: 2026-03-09T20:19:40
---

# Story: Detect Scanned PDFs And Fallback To OCR

## Acceptance Criteria

- [x] [SRS-01/AC-01] Implement `is_scanned_pdf_heuristic` to detect empty or sparse text <!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-01] Refactor OCR logic into a reusable `perform_ocr` helper function <!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-02] Update `extract_pdf` to write bytes to a temp file and call `perform_ocr` if heuristic triggers <!-- verify: manual, SRS-02:start:end, proof: ac-3.log -->
- [x] [SRS-03/AC-01] Ensure `extract_pdf` logs a warning and returns the sparse text if the `ocr` feature is disabled <!-- verify: manual, SRS-03:start:end, proof: ac-4.log -->
- [x] [SRS-04/AC-01] Verify no latency regressions for standard PDFs by avoiding OCR calls for text-heavy documents <!-- verify: manual, SRS-04:start:end, proof: ac-5.log -->

## Evidence
- **manual**: Inspect `src/extract.rs` for the heuristic and fallback logic.
