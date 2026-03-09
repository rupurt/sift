---
id: 1vzfew000
title: Zig-Style Global File Cache
created_at: 2026-03-09T11:52:42
---

# Zig-Style Global File Cache

> sift currently extracts text and computes dense embeddings (which is expensive) for every file on every run. As the corpus grows, this transient approach becomes a massive performance bottleneck. We need a way to reuse extraction and vectorization work across runs and across different projects, but we must strictly avoid the complexity of traditional sidecar databases or daemons.

## Documents

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Product requirements and success criteria |
| `PRESS_RELEASE.md` (optional) | Working-backwards artifact for large user-facing launches; usually skip for incremental/refactor/architecture-only work |

## Voyages

<!-- BEGIN GENERATED -->
**Progress:** 0/1 voyages complete, 0/3 stories done
| Voyage | Status | Stories |
|--------|--------|---------|
| [Incremental File Caching](voyages/1vzfjD000/) | in-progress | 0/3 |
<!-- END GENERATED -->
