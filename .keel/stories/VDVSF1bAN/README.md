---
id: VDVSF1bAN
title: Adopt The Facade In The Executable
type: feat
status: backlog
created_at: 2026-03-10T15:30:27
updated_at: 2026-03-10T15:31:02
scope: VDVQurZER/VDVRkNjgH
index: 3
---

# Adopt The Facade In The Executable

## Summary

Rewire `src/main.rs` to consume the curated library facade while preserving the
current executable command surface and help contract.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] The executable builds and routes search execution through the curated library boundary instead of depending on newly unsupported internals. <!-- verify: cargo test && cargo run -- --help && cargo run -- search --help, SRS-03:start:end -->
- [ ] [SRS-03/AC-02] User-facing command names, argument shapes, and baseline behavior remain intact across the cutover. <!-- verify: cargo run -- --help && cargo run -- search --help, SRS-03:start:end -->
