---
created_at: 2026-03-08T15:54:47
---

# Reflection - Add Text-Bearing PDF Extraction

## Knowledge

- [1vzMmS000](../../knowledge/1vzMmS000.md) Add Formats Below The Loader Boundary

## Observations

This story stayed small because the extractor boundary from the HTML slice was
already in place. PDF support reduced to one new handler, one fixture, and one
focused search test instead of another search-layer refactor.

The most delicate part was the fixture, not the extraction code. A minimal
text-bearing PDF with correct xref offsets was enough for `pdf-extract` to
exercise the real path in both unit tests and CLI proofs, which avoided pulling
in a larger binary sample just to validate the integration.

The architecture constraint held cleanly here. `pdf-extract` runs entirely
in-process, so the story could satisfy the no-daemon requirement without adding
special runtime handling above the existing loader seam.
