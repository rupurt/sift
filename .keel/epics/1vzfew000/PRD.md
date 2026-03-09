# Zig-Style Global File Cache - Product Requirements

> By adopting a Zig-style incremental caching model (hashing at the file level and looking up cached metadata via fast filesystem heuristics like `mtime`, `inode`, and `size`), we can build a lightning-fast "Search Asset Pipeline" without the operational overhead of a daemon or full relational database.

## Problem Statement

`sift` currently extracts text and computes dense embeddings (which is expensive) for every file on every run. As the corpus grows, this transient approach becomes a massive performance bottleneck. We need a way to reuse extraction and vectorization work across runs and across different projects, but we must strictly avoid the complexity of traditional sidecar databases or daemons (like the removed `zvec`).

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Validate bearing recommendation in delivery flow | Adoption signal | Initial rollout complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Product/Delivery Owner | Coordinates planning and execution | Reliable strategic direction |

## Scope

### In Scope

- Deliver the bearing-backed capability slice for this epic.

### Out of Scope

- Unrelated platform-wide refactors outside bearing findings.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Implement the core user workflow identified in bearing research. | GOAL-01 | must | Converts research recommendation into executable product capability. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Ensure deterministic behavior and operational visibility for the delivered workflow. | GOAL-01 | must | Keeps delivery safe and auditable during rollout. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove functional behavior through story-level verification evidence mapped to voyage requirements.
- Validate non-functional posture with operational checks and documented artifacts.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Bearing findings reflect current user needs | Scope may need re-planning | Re-check feedback during first voyage |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which rollout constraints should gate broader adoption? | Product | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Define the storage format and metadata lookup mechanisms that avoid SQLite/databases while maintaining fast cache hits.
- [ ] Determine how to uniquely identify a file state (e.g., `(inode, mtime, size) -> blake3_hash`) safely and globally.
- [ ] Ensure the approach supports `mmap` for fast reading of pre-computed embeddings and term frequencies.
- [ ] Validate the alignment with `sift`'s core principles (Zero-Friction Operations, Determinism).
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

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

---

*This PRD was seeded from bearing `1vzfew000`. See `bearings/1vzfew000/` for original research.*
