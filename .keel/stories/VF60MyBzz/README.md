---
id: VF60MyBzz
title: Expose Simplified Artifact Runtime Through CLI And Public Facade
type: feat
status: done
created_at: 2026-03-27T12:00:41
updated_at: 2026-03-27T13:21:36
operator-signal: 
scope: VF53suDXv/VF60I1J0l
index: 3
started_at: 2026-03-27T13:19:32
submitted_at: 2026-03-27T13:21:27
completed_at: 2026-03-27T13:21:36
---

# Expose Simplified Artifact Runtime Through CLI And Public Facade

## Summary

Expose the simplified artifact-based runtime through supported CLI and library
surfaces while preserving the current single-turn hybrid UX when the new path
is not selected.

## Acceptance Criteria

- [x] [SRS-04/AC-01] Supported CLI and library entry points can invoke the simplified artifact runtime without graph or Reactor concepts. <!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
- [x] [SRS-05/AC-02] Supported runtime surfaces operate over `ContextArtifact` without a supported `Document` compatibility sidecar. <!-- verify: manual, SRS-05:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-02/AC-03] The current single-turn hybrid experience remains available when the artifact runtime path is not selected. <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->
