# Zig-Style Global File Cache — Evidence & Survey

## Overview

We need a caching strategy that relies on file system heuristics (like Zig) rather than a persistent sidecar daemon or database process. This allows `sift` to be as fast as a compiler while retaining the zero-setup CLI experience.

## Feasibility

Heuristic-based caching is highly feasible in Rust using standard library `std::fs::Metadata` and the `blake3` crate for fast hashing [SRC-01]. Advisory file locking via `fs4` ensures safe concurrent access to global cache manifests [SRC-02].

## Key Findings

- Zig's model of mapping `(mtime, size, inode)` to a content hash allows for O(1) change detection on subsequent runs [SRC-01].
- Decoupling the Metadata Store from the Content-Addressable Storage (CAS) allows the same blob to be reused across different projects [SRC-01].
- Maintaining a "database-free" architecture requires partitioning manifests by project path to avoid global write contention [SRC-02].

## Unknowns

- How will `inode` stability behave across different filesystems (e.g., networked drives or Docker mounts)?
- What is the performance impact of loading very large (1GB+) project manifests into memory?

## Analysis of Zig's Caching Model

Zig achieves incredibly fast incremental builds without databases by leveraging the file system directly:
1.  **Fast Path (Heuristics):** Zig looks at file `mtime` (modification time), `inode`, and `size`. If these match the last known state, it assumes the file hasn't changed.
2.  **Manifest/Index Files:** It maintains simple binary or text manifest files that map these heuristics to a strong cryptographic hash (or directly to the output object).
3.  **Content-Addressable Storage (CAS):** The actual cached object is stored in a global cache directory named by its hash. If two projects use the exact same file, it is only compiled once.

## Applying the Model to Sift

### 1. Global Asset Pipeline (`~/.cache/sift/blobs/`)
We will store the expensive computed data here. The key is the **BLAKE3 hash** of the file's content.
*   `~/.cache/sift/blobs/<blake3-hash>`
*   Format: A simple, fast binary format like `bincode` or `rkyv`. This file will contain:
    *   `text`: The extracted plain text string.
    *   `term_frequencies`: A pre-computed `HashMap<String, usize>` or similar for BM25.
    *   `segment_tensors`: The dense embeddings for each segment, stored in a layout that can be loaded into `candle` via `mmap`.

### 2. The Heuristic Index (The Metadata Store)
We must avoid a database like SQLite to adhere to our "no database" rule, but we need a fast way to map `(path, size, mtime, inode)` to `<blake3-hash>`.

**Option A: Global Flat File / Binary Hash Map**
Store a global index at `~/.cache/sift/state/index.bin`.
*   *Pros:* One place for all metadata.
*   *Cons:* Concurrent writes from multiple `sift` invocations (e.g., in parallel agent workflows) require file locks, which can become a bottleneck. Absolute paths must be used, which are brittle if folders are moved.

**Option B: Per-Project `.sift/cache-manifest`**
Store a lightweight manifest file at the root of the searched path.
*   *Pros:* No global concurrency issues. Paths can be relative to the manifest. Extremely fast to load just the relevant subset.
*   *Cons:* Leaves an artifact in the user's project directory (violates pure "stateless/untouched" philosophy).

**Option C: Global Directory of Manifests by Path Hash**
Store manifests in `~/.cache/sift/manifests/<hash-of-absolute-path>.bin`.
*   *Pros:* Keeps the user's directory untouched. Minimizes concurrency issues because each searched directory gets its own manifest file.
*   *Cons:* If a folder is moved, it misses the cache initially (though the global CAS will quickly hit once files are blake3-hashed).

### Decision for Metadata Store: Option C

Option C gives us the best of both worlds:
1.  We maintain the "untouched project directory" rule.
2.  We avoid a single global file lock by partitioning manifests by the absolute path of the search root.
3.  Inside each manifest, we map relative paths (plus their `mtime`, `inode`, `size`) to their `blake3` hash.
4.  If the manifest says the file hasn't changed, we grab the hash. We then `mmap` the blob from `~/.cache/sift/blobs/<hash>`.

## Memory Mapping (mmap)
To make this blazing fast, the `blobs` must be designed for `mmap`. `candle` already supports `mmap` for SafeTensors. We can serialize our `Document` structures in a way that allows zero-copy reading of strings and tensors. However, for the first pass, simply using `bincode` and standard file reading will provide a massive speedup over re-extracting PDFs and running neural network inference.

## Concurrency
We will need advisory file locks (e.g., using the `fs4` or `fd-lock` crate) when writing to a manifest or a blob to prevent corruption if two `sift` processes run on the same directory simultaneously.

## Toolchain Verification
With `zvec` removed, `sift` now compiles purely with Rust. We can confidently target `x86_64-unknown-linux-musl` for a fully static executable.

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | web | Zig Compiler | https://ziglang.org | 2024-01-01 | 2026-03-09 | high | high | Heuristic-based file caching without DBs. |
| SRC-02 | manual | Sift Constitution | Internal | 2026-03-09 | 2026-03-09 | high | high | Confirms stateless and database-free constraints. |
