---
id: 1vzdE0000
title: Add Named Hybrid Strategy Presets
type: feat
status: backlog
created_at: 2026-03-09T09:12:56
updated_at: 2026-03-09T09:20:48
scope: 1vzXLN000/1vzdCx000
index: 8
---

# Add Named Hybrid Strategy Presets

## Summary

Add a registry of named strategy presets so sift can expose baseline, hybrid,
and PageIndex-inspired search compositions while keeping `hybrid` as a stable
alias to the current champion preset.

## Acceptance Criteria

- [ ] [SRS-09/AC-01] Sift ships named strategy presets including `bm25`, at
      least one composite hybrid preset, and one PageIndex-inspired preset
      definition composed from the available layers.
- [ ] [SRS-10/AC-02] The top-level `hybrid` strategy resolves through a
      configurable champion preset instead of embedding retrieval logic directly
      in the CLI command path.
- [ ] [SRS-13/AC-03] The shipped presets preserve the default single-binary,
      local-first operating contract.
