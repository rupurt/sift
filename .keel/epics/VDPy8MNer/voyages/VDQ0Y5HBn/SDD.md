# Mmap-Accelerated Blob Retrieval - SDD

> Implement memory-mapped I/O for document blob retrieval to improve search performance.

## Architecture Overview

We will introduce a memory-mapped I/O layer for retrieving documents from the global blob store. This reduces the number of context switches and allows the OS to handle caching more effectively.

## Components

### `load_blob` (Cache)
Updated to use `memmap2::Mmap` to map the document blob from disk and deserialize it from the memory slice using `bincode`.

## Data Flow

1. **Retrieval Start:** A document is requested from the cache by its hash.
2. **Memory Mapping:** `load_blob` opens the file and creates a `memmap2::Mmap` object.
3. **Deserialization:** `bincode::deserialize_from` is called using the mapped slice.
4. **Clean up:** The mapped memory is automatically unmapped when the `Mmap` object goes out of scope.

## Design Approach

### Mmap Integration

We will use the `memmap2` crate to map the file into memory. 
- **Mapping:** Use `Mmap::map(&file)` to create a read-only memory mapping.
- **Deserialization:** Use `bincode::deserialize(&mmap)` to deserialize from the slice.

## Deployment Strategy

- Incrementally apply changes to `src/cache/store.rs`.
- Verify performance after each change using `sift search -vv`.
- Run micro-benchmarks to quantify the speedup.
