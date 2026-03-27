---
id: VF60MwivG
title: Hard-Cut Corpus Storage IDs And Segmentation To Artifact Semantics
type: feat
status: done
created_at: 2026-03-27T12:00:41
updated_at: 2026-03-27T13:21:36
operator-signal: 
scope: VF53stqXt/VF60I0r0f
index: 2
started_at: 2026-03-27T13:19:19
submitted_at: 2026-03-27T13:21:27
completed_at: 2026-03-27T13:21:36
---

# Hard-Cut Corpus Storage IDs And Segmentation To Artifact Semantics

## Summary

Convert corpus/storage identity and segmentation rules to artifact-native
semantics so caching, deduplication, and retrieval no longer depend on
document-specific assumptions.

## Acceptance Criteria

- [x] [SRS-03/AC-01] Stable artifact and segment identity rules are defined for the shared local corpus substrate. <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-02] The hard cutover does not leave a supported `Document` compatibility surface behind in corpus and storage contracts. <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-2.log-->
