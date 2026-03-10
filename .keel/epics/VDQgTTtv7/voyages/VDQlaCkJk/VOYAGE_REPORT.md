# VOYAGE REPORT: Implement Scanned PDF Fallback

## Voyage Metadata
- **ID:** VDQlaCkJk
- **Epic:** VDQgTTtv7
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Detect Scanned PDFs And Fallback To OCR
- **ID:** VDQlp68MV
- **Status:** done

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Implement `is_scanned_pdf_heuristic` to detect empty or sparse text <!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-01] Refactor OCR logic into a reusable `perform_ocr` helper function <!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-02] Update `extract_pdf` to write bytes to a temp file and call `perform_ocr` if heuristic triggers <!-- verify: manual, SRS-02:start:end, proof: ac-3.log -->
- [x] [SRS-03/AC-01] Ensure `extract_pdf` logs a warning and returns the sparse text if the `ocr` feature is disabled <!-- verify: manual, SRS-03:start:end, proof: ac-4.log -->
- [x] [SRS-04/AC-01] Verify no latency regressions for standard PDFs by avoiding OCR calls for text-heavy documents <!-- verify: manual, SRS-04:start:end, proof: ac-5.log -->

#### Implementation Insights
- **VDQmKRtRl: Scanned PDF Heuristic**
  - Insight: Counting alphanumeric characters (`< 50`) is a reliable, fast heuristic for identifying image-only PDFs before falling back to computationally expensive OCR.
  - Suggested Action: Use simple text length heuristics as a gatekeeper for heavy processing steps like OCR.
  - Applies To: src/extract.rs
  - Category: code


#### Verified Evidence
- [ac-4.log](../../../../stories/VDQlp68MV/EVIDENCE/ac-4.log)
- [ac-5.log](../../../../stories/VDQlp68MV/EVIDENCE/ac-5.log)
- [ac-1.log](../../../../stories/VDQlp68MV/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VDQlp68MV/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VDQlp68MV/EVIDENCE/ac-3.log)


