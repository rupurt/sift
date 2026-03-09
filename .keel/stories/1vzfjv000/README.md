---
id: 1vzfjv000
title: Implement Blob Store And Manifest Logic
type: feat
status: backlog
scope: 1vzfew000/1vzfjD000
created_at: 2026-03-09T11:21:45
updated_at: 2026-03-09T11:54:55
---

# Implement Blob Store And Manifest Logic

## Context

We need the logic to read/write from `~/.cache/sift/blobs/` and `~/.cache/sift/manifests/`.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Implement `hash_file` function using `blake3`.
- [ ] [SRS-03/AC-01] Implement `Manifest::load` and `Manifest::save` using `fs2` for advisory locking.
- [ ] [SRS-04/AC-01] Implement heuristic matching logic checking `(inode, mtime, size)`.
- [ ] [SRS-05/AC-01] Use file locking via `fs2` during manifest writing.
