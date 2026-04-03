---
# system-managed
id: VFkKHRGx5
status: done
created_at: 2026-04-03T09:30:40
updated_at: 2026-04-03T09:39:20
# authored
title: Add Crate-Root Model Preparation Seam
type: feat
operator-signal:
scope: VFkJD2NwK/VFkJcKSVA
index: 1
started_at: 2026-04-03T09:31:25
submitted_at: 2026-04-03T09:39:18
completed_at: 2026-04-03T09:39:20
---

# Add Crate-Root Model Preparation Seam

## Summary

Add a stable crate-root API that prepares local model artifacts for the current
Candle-loadable bundle contract, reuses compatible bundles when possible,
routes GGUF conversion through metamorph when necessary, updates the existing
local Candle loaders to reuse the seam, and documents the result as a
compatibility path rather than native 1-bit execution support.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Crate-root model-preparation types and a `prepare_model` entry point are exported as supported library surface. <!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-02/AC-02] The preparation API accepts local and `hf://repo[@revision]` sources and returns validated local bundle paths for directly compatible inputs. <!-- verify: manual, SRS-02:start:end -->
- [x] [SRS-03/AC-03] A representative GGUF source can be prepared through metamorph into a validated Candle-loadable bundle with compatibility metadata. <!-- verify: manual, SRS-03:start:end -->
- [x] [SRS-04/AC-04] Qwen/Gemma/Jina local model loaders reuse the new seam instead of owning direct HF asset preparation. <!-- verify: manual, SRS-04:start:end -->
- [x] [SRS-05/AC-05] README and LIBRARY docs explain the seam and state explicitly that GGUF conversion is a compatibility path, not native 1-bit execution support. <!-- verify: manual, SRS-05:start:end -->
