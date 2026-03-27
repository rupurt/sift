---
id: VF1Vxv5U1
title: Emit Inspectable Turn Traces And Context Actions
type: feat
status: icebox
created_at: 2026-03-26T17:34:50
updated_at: 2026-03-26T17:40:53
operator-signal: 
scope: VF1VhyGqK/VF1Vt0sCV
index: 1
---

# Emit Inspectable Turn Traces And Context Actions

## Summary

Emit explicit turn traces and context actions so controller behavior can be inspected without relying on log spelunking.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Multi-turn runs emit per-turn trace records. <!-- verify: manual, SRS-01:start:end -->
- [ ] [SRS-NFR-01/AC-02] Trace artifacts are deterministic enough for replay or regression review. <!-- verify: manual, SRS-NFR-01:start:end -->
