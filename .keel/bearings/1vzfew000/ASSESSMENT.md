# Zig-Style Global File Cache — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | Drastically reduces latency for repeat queries (seconds to milliseconds). |
| Confidence | 4 | Zig has proven this model at scale. File locking and `bincode` are well-understood in Rust. |
| Effort | 3 | Requires restructuring `PreparedCorpus` to build around cache logic. |
| Risk | 2 | The primary risk is cache invalidation bugs leading to stale results. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Opportunity Cost

The main opportunity cost is the time spent building binary serialization and file locks instead of adding more retrieval capabilities. This is an acceptable trade because the current system cannot scale to larger repos without a massive slowdown.

### Findings

- Heuristic caching using mtime, size, and inode works well for compilers without a database [SRC-01]
- `sift` is strictly constrained from using background db daemons, meaning caching must be file-based [SRC-02]

### Dependencies

The following must hold:

- we can quickly compute file hashes, likely using the `blake3` crate [SRC-01]
- we can safely write cache manifests without corrupting state during concurrent runs [SRC-02]

### Alternatives Considered

Alternatives considered:

- Global Flat File Index.
  Rejected because concurrent writes require global file locks, becoming a bottleneck. [SRC-01]
- Per-Project `.sift/cache-manifest`.
  Rejected because it violates the zero-friction constraint by writing artifacts to the user's project directory. [SRC-02]
- Global Directory of Manifests by Path Hash.
  Recommended. Keeps user directory untouched and minimizes global locking conflicts. [SRC-01]

## Recommendation

- [x] Proceed → convert to epic [SRC-01] [SRC-02]
- [ ] Park → revisit later
- [ ] Decline → document learnings

Proceed with a new epic that introduces the "Incremental Caching Pipeline" with:
1. `blake3` and `bincode` serialization for extracted/embedded assets.
2. A global Blob Store in `~/.cache/sift/blobs/`.
3. A global manifest directory hashing project paths.
