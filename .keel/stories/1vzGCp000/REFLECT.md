---
created_at: 2026-03-08T09:18:29
---

# Reflection - Stabilize Zvec Build Toolchain

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzGpp000: Title
| Field | Value |
|-------|-------|
| **Category** | code/testing/process/architecture |
| **Context** | describe when this applies |
| **Insight** | the fundamental discovery |
| **Suggested Action** | what to do next time |
| **Applies To** | file patterns or components |
| **Linked Knowledge IDs** | optional canonical IDs this insight builds on |
| **Observed At** | RFC3339 timestamp (e.g. 2026-02-22T12:00:00Z) |
| **Score** | 0.0-1.0 (impact significance) |
| **Confidence** | 0.0-1.0 (insight quality) |
| **Applied** | |
-->

## Observations

- The original CMake failure was only the first layer of the problem. Once the
  build used CMake 3.x, GCC 15 exposed a second incompatibility in vendored
  RocksDB headers, so the durable fix needed both a CMake pin and a compiler
  pin.
- Vendoring `zvec-sys` was the smallest reliable patch surface. A shell-only
  change still failed because the upstream wrapper did not match the
  `zvec v0.2.0` `Collection::AddColumn` signature.
- Isolating the native build under Cargo `OUT_DIR` avoided stale shared
  `vendor/zvec/build` state in the cargo registry and made repeat verification
  deterministic.
