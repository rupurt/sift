---
id: VDVSF1KAM
title: Decouple Public API From CLI Concerns
type: feat
status: backlog
created_at: 2026-03-10T15:30:27
updated_at: 2026-03-10T15:31:02
scope: VDVQurZER/VDVRkNjgH
index: 2
---

# Decouple Public API From CLI Concerns

## Summary

Narrow the supported library API so it no longer depends on CLI parsing,
terminal rendering, or other executable-only concerns in public types.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] Supported public library types used by embedders do not require `clap` derives or CLI-only enums in their contract. <!-- verify: cargo test, SRS-02:start:end -->
- [ ] [SRS-02/AC-02] Terminal rendering helpers and similar CLI presentation concerns are no longer part of the canonical embedded API path. <!-- verify: cargo test, SRS-02:start:end -->
