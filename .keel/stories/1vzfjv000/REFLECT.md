# Reflection - Implement Blob Store And Manifest Logic

Implemented the core data storage mechanism:
- `hash_file` uses `blake3` for fast, cryptographic content addressing.
- `Manifest` now uses `fs2` to establish advisory locks (shared for reading, exclusive for writing) to ensure safe concurrent operations from multiple `sift` instances.
- Added `get_file_heuristics` to capture inode, mtime, and size directly from the OS.
