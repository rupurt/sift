# Incremental File Caching - Software Design Description

> Implement the manifest and blob storage logic to bypass document extraction.

## Architecture

We are introducing two new core concepts to the Domain and Application layers:
1. **Blob Store:** A global directory (`~/.cache/sift/blobs/`) storing `Document` representations serialized with `bincode`. The filename is the `blake3` hash of the original file.
2. **Manifest Store:** A global directory (`~/.cache/sift/manifests/`) containing project-level metadata. The filename is the `blake3` hash of the absolute directory path being searched.

When `corpus.rs` processes a file, it will now pass through the cache:
```rust
let heuristics = get_file_heuristics(path);
let manifest = load_manifest(project_path_hash);

if let Some(hash) = manifest.check_heuristics(path, &heuristics) {
    if let Ok(document) = load_blob(hash) {
        return document;
    }
}

// Cache miss path
let hash = blake3_hash_file(path);
if let Ok(document) = load_blob(hash) {
    manifest.update(path, heuristics, hash);
    return document;
}

// True miss path
let document = extract_and_embed(path);
save_blob(hash, &document);
manifest.update(path, heuristics, hash);
return document;
```

## Data Models

```rust
#[derive(Serialize, Deserialize)]
pub struct Manifest {
    pub entries: HashMap<PathBuf, CacheEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct CacheEntry {
    pub inode: u64,
    pub mtime_secs: i64,
    pub mtime_nanos: u32,
    pub size: u64,
    pub blake3_hash: String,
}
```

## Dependencies
We will add `blake3`, `bincode`, and `fs2` (or `fs4`) to `Cargo.toml`.
