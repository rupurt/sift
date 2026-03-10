---
created_at: 2026-03-09T20:19:40
---

# Knowledge - VDQlaCkJk

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Detect Scanned PDFs And Fallback To OCR (VDQlp68MV)

### VDQmKRtRl: Scanned PDF Heuristic

| Field | Value |
|-------|-------|
| **Category** | code |
| **Context** | Processing PDFs without a text layer. |
| **Insight** | Counting alphanumeric characters (`< 50`) is a reliable, fast heuristic for identifying image-only PDFs before falling back to computationally expensive OCR. |
| **Suggested Action** | Use simple text length heuristics as a gatekeeper for heavy processing steps like OCR. |
| **Applies To** | src/extract.rs |
| **Applied** | true |



---

## Synthesis

### 49FWdeZrR: Scanned PDF Heuristic

| Field | Value |
|-------|-------|
| **Category** | code |
| **Context** | Processing PDFs without a text layer. |
| **Insight** | Counting alphanumeric characters (`< 50`) is a reliable, fast heuristic for identifying image-only PDFs before falling back to computationally expensive OCR. |
| **Suggested Action** | Use simple text length heuristics as a gatekeeper for heavy processing steps like OCR. |
| **Applies To** | src/extract.rs |
| **Linked Knowledge IDs** | VDQmKRtRl |
| **Score** | 0.80 |
| **Confidence** | 0.90 |
| **Applied** | true |

