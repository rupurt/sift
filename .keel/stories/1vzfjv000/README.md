---
id: 1vzfjv000
title: Implement Blob Store And Manifest Logic
type: feat
status: done
scope: 1vzfew000/1vzfjD000
created_at: 2026-03-09T11:21:45
updated_at: 2026-03-09T11:58:41
started_at: 2026-03-09T11:58:01
submitted_at: 2026-03-09T11:58:30
completed_at: 2026-03-09T11:58:41
---

# Implement Blob Store And Manifest Logic

## Context

We need the logic to read/write from `~/.cache/sift/blobs/` and `~/.cache/sift/manifests/`.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Implement `hash_file` function using `blake3`. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-03/AC-01] Implement `Manifest::load` and `Manifest::save` using `fs2` for advisory locking. <!-- verify: manual, SRS-03:start:end, proof: ac-2.log -->
- [x] [SRS-04/AC-01] Implement heuristic matching logic checking `(inode, mtime, size)`. <!-- verify: manual, SRS-04:start:end, proof: ac-3.log -->
- [x] [SRS-05/AC-01] Use file locking via `fs2` during manifest writing. <!-- verify: manual, SRS-05:start:end, proof: ac-4.log -->
