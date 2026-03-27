---
id: VF1VvuNO5
title: Make Search Plans Authoritative For Controller Execution
type: feat
status: backlog
created_at: 2026-03-26T17:34:42
updated_at: 2026-03-26T17:39:53
operator-signal: 
scope: VF1Vhy2qJ/VF1Vt0iCU
index: 1
---

# Make Search Plans Authoritative For Controller Execution

## Summary

Remove or contain implicit execution overrides so controller behavior can be driven from explicit plan and state data.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] A deterministic multi-turn execution path can drive retrieval from explicit controller state. <!-- verify: manual, SRS-01:start:end -->
- [ ] [SRS-02/AC-02] Controller execution relies on plan or state data rather than hidden runtime overrides. <!-- verify: manual, SRS-02:start:end -->
- [ ] [SRS-NFR-02/AC-03] The single-turn hybrid path is preserved when the controller is not selected. <!-- verify: manual, SRS-NFR-02:start:end -->
