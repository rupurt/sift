---
id: VF60MwXtc
title: Replace Document-Centric Search Types With ContextArtifact
type: feat
status: done
created_at: 2026-03-27T12:00:41
updated_at: 2026-03-27T13:21:36
operator-signal: 
scope: VF53stqXt/VF60I0r0f
index: 1
started_at: 2026-03-27T12:56:22
submitted_at: 2026-03-27T13:21:27
completed_at: 2026-03-27T13:21:36
---

# Replace Document-Centric Search Types With ContextArtifact

## Summary

Replace the primary search-domain records that still assume `Document` with
artifact-native types so the local substrate has one canonical unit of context.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `ContextArtifact`-native records exist for the initial strictly local context kinds. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-02] Core search-domain interfaces stop treating `Document` as a supported parallel primary type. <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-02/AC-03] The cutover does not introduce new graph or engine abstractions beyond the artifact substrate. <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->
