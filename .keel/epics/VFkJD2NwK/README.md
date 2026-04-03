---
# system-managed
id: VFkJD2NwK
created_at: 2026-04-03T09:26:25
# authored
title: Stable Local Model Preparation Seam
index: 23
mission: VFkJD1xwQ
---

# Stable Local Model Preparation Seam

> Downstream embedders currently reach into sift internals to acquire or prepare local model artifacts. Newly published runtime-specific formats like GGUF and MLX deepen that leak because sift has no stable public preparation boundary that can reuse compatible bundles or invoke metamorph when conversion is required.

## Documents

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Product requirements and success criteria |
| `PRESS_RELEASE.md` (optional) | Working-backwards artifact for large user-facing launches; usually skip for incremental/refactor/architecture-only work |

## Voyages

<!-- BEGIN GENERATED -->
**Progress:** 1/1 voyages complete, 1/1 stories done
| Voyage | Status | Stories |
|--------|--------|---------|
| [Expose Stable Model Preparation API](voyages/VFkJcKSVA/) | done | 1/1 |
<!-- END GENERATED -->
