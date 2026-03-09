# Constitution

This document defines the unyielding principles that guide the development and design of `sift`. When making architectural decisions, evaluating PRs, or planning new features, these rules must be satisfied.

## 1. Zero-Friction Operations
`sift` is a CLI tool, not a service stack.
- **No Daemons:** There will be no long-running background processes or resident services.
- **No Databases:** We do not require users to install, configure, or manage external databases (e.g., Postgres, Redis, external Vector DBs).
- **Stateless UX:** From the user's perspective, `sift` operates on directories immediately. Any caching or indexing must happen transparently in standard user cache directories (e.g., `~/.cache/sift`) without requiring explicit lifecycle management commands (`sift start`, `sift stop`).

## 2. Fast Heuristic Caching
`sift` follows the Zig-style "Search Asset Pipeline" model.
- **File-Level Heuristics:** We use filesystem metadata (`mtime`, `inode`, `size`) to avoid redundant work (text extraction, hashing, embedding) on unchanged files.
- **Content-Addressable Storage:** Cached assets are keyed by BLAKE3 content hashes for cross-project deduplication.
- **Advisory Locking:** All cache manifests are protected by filesystem advisory locks to ensure safe concurrent operations from multiple `sift` processes.

## 3. Pure Rust & Static Binaries
`sift` must remain easily distributable.
- **No C++ Toolchains:** We strictly avoid C++ dependencies (like RocksDB, ProtoBuf, or Arrow) to ensure a clean Rust-only build.
- **Static Distribution:** `sift` must be capable of building as a fully static executable for easy installation without shared library conflicts.

## 4. Determinism and Traceability
Search results and evaluations must be reproducible.
- Tie-breaking in ranking must be stable (e.g., falling back to lexicographical path sorting).
- File tree traversal must be deterministic.
- Benchmarks must record the exact state of the world: git SHA, command used, corpus size, model parameters, and hardware environment.

## 5. Composition over Monoliths
Search is a pipeline, not a single algorithm.
- We do not hardcode "hybrid" as a single function. We compose it via `Query Expansion -> Retrieval -> Fusion -> Reranking`.
- Strategies are defined as data (Presets/Plans) and executed by an orchestrator, allowing for rapid experimentation and objective benchmarking.

## 6. Local First
`sift` is built for local development and agentic workflows.
- Code stays on the machine. We do not send source code or documents to external APIs for embedding or search by default.
- Machine learning models run locally via pure-Rust implementations (e.g., `candle`), utilizing CPU and local accelerators.

## 7. Domain-Driven Isolation
The core search logic must remain pure.
- The terminal (CLI arguments, printing), the filesystem (walking directories), and the network (downloading datasets) must stay at the edge of the architecture.
- Domain models and ports (`Retriever`, `Fuser`) define the behavior; adapters implement the details.

## 8. Verification-Driven Delivery
Implementation does not end at a clean compile.
- Every functional change must be verified against a test, a benchmark, or an empirical CLI proof.
- If a change degrades the benchmark quality against the BM25 baseline or the champion preset, the change must be justified with evidence.
