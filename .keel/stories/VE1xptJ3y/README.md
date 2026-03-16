---
id: VE1xptJ3y
title: Update Expanders to Use Configured Prompts
type: feat
status: done
created_at: 2026-03-16T04:56:39
updated_at: 2026-03-16T05:16:15
operator-signal: 
scope: VE1xUOaxK/VE1xdk4hF
index: 2
started_at: 2026-03-16T05:05:04
submitted_at: 2026-03-16T05:16:11
completed_at: 2026-03-16T05:16:15
---

# Update Expanders to Use Configured Prompts

## Summary

This story refactors the `SearchServiceBuilder` and expansion strategies to utilize the prompts loaded from the configuration, making the expansion process dynamic.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Update `SearchServiceBuilder` to read prompts from config. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Refactor `HydeStrategy`, `SpladeStrategy`, and `ClassifiedStrategy` to accept configurable prompts. <!-- verify: manual, SRS-01:start:end, proof: ac-2.log -->
