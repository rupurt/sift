---
id: VE1xpmsiN
title: Add Prompt Configuration to Sift Toml
type: feat
status: done
created_at: 2026-03-16T04:56:39
updated_at: 2026-03-16T05:10:23
operator-signal: 
scope: VE1xUOaxK/VE1xdk4hF
index: 1
started_at: 2026-03-16T05:05:31
submitted_at: 2026-03-16T05:10:07
completed_at: 2026-03-16T05:10:23
---

# Add Prompt Configuration to Sift Toml

## Summary

This story adds the `[prompts]` section to the `sift.toml` configuration structure, enabling users to define custom prompts for generative query expansion strategies.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Add `prompts` section to `Config` struct. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Parse prompts correctly from `sift.toml`. <!-- verify: manual, SRS-01:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-03] Provide default fallback constants. <!-- verify: manual, SRS-02:start:end, proof: ac-3.log -->
