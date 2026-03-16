---
id: VE1xpmsiN
title: Add Prompt Configuration to Sift Toml
type: feat
status: backlog
created_at: 2026-03-16T04:56:39
updated_at: 2026-03-16T04:59:37
operator-signal: 
scope: VE1xUOaxK/VE1xdk4hF
index: 1
---

# Add Prompt Configuration to Sift Toml

## Summary

This story adds the `[prompts]` section to the `sift.toml` configuration structure, enabling users to define custom prompts for generative query expansion strategies.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Add `prompts` section to `Config` struct. <!-- verify: manual -->
- [ ] [SRS-01/AC-02] Parse prompts correctly from `sift.toml`. <!-- verify: manual -->
- [ ] [SRS-02/AC-03] Provide default fallback constants. <!-- verify: manual -->
