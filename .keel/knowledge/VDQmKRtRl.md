---
source_type: Story
source: stories/VDQlp68MV/REFLECT.md
scope: VDQgTTtv7/VDQlaCkJk
source_story_id: VDQlp68MV
created_at: 2026-03-09T20:18:53
---

### VDQmKRtRl: Scanned PDF Heuristic

| Field | Value |
|-------|-------|
| **Category** | code |
| **Context** | Processing PDFs without a text layer. |
| **Insight** | Counting alphanumeric characters (`< 50`) is a reliable, fast heuristic for identifying image-only PDFs before falling back to computationally expensive OCR. |
| **Suggested Action** | Use simple text length heuristics as a gatekeeper for heavy processing steps like OCR. |
| **Applies To** | src/extract.rs |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-09T20:25:00+00:00 |
| **Score** | 0.80 |
| **Confidence** | 0.90 |
| **Applied** | true |
