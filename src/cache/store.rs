use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use fs2::FileExt;

use crate::cache::model::{CacheEntry, Manifest};
use crate::search::domain::Document;

pub fn hash_file(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut hasher = blake3::Hasher::new();
    std::io::copy(&mut file, &mut hasher)
        .with_context(|| format!("failed to hash {}", path.display()))?;
    Ok(hasher.finalize().to_string())
}

pub fn get_file_heuristics(path: &Path) -> Result<CacheEntry> {
    use std::os::unix::fs::MetadataExt;

    let meta = fs::metadata(path).with_context(|| format!("failed to stat {}", path.display()))?;
    Ok(CacheEntry {
        inode: meta.ino(),
        mtime_secs: meta.mtime(),
        mtime_nanos: meta.mtime_nsec() as u32,
        size: meta.size(),
        blake3_hash: String::new(), // Will be populated if needed
    })
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let file = File::open(path)
            .with_context(|| format!("failed to open manifest {}", path.display()))?;
        file.lock_shared()
            .context("failed to acquire shared lock on manifest")?;

        let manifest: Manifest = bincode::deserialize_from(&file).unwrap_or_default();

        file.unlock()
            .context("failed to release shared lock on manifest")?;
        Ok(manifest)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("failed to create manifest directory")?;
        }

        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .with_context(|| format!("failed to open manifest for writing {}", path.display()))?;

        file.lock_exclusive()
            .context("failed to acquire exclusive lock on manifest")?;

        bincode::serialize_into(&file, self)
            .with_context(|| format!("failed to serialize manifest to {}", path.display()))?;

        file.unlock()
            .context("failed to release exclusive lock on manifest")?;
        Ok(())
    }

    pub fn check_heuristics(&self, path: &Path, current: &CacheEntry) -> Option<String> {
        if let Some(cached) = self.entries.get(path)
            && cached.inode == current.inode
            && cached.mtime_secs == current.mtime_secs
            && cached.mtime_nanos == current.mtime_nanos
            && cached.size == current.size
        {
            return Some(cached.blake3_hash.clone());
        }
        None
    }

    pub fn update(&mut self, path: PathBuf, mut entry: CacheEntry, hash: String) {
        entry.blake3_hash = hash;
        self.entries.insert(path, entry);
    }
}

pub fn save_blob(blobs_dir: &Path, hash: &str, document: &Document) -> Result<()> {
    fs::create_dir_all(blobs_dir).context("failed to create blobs directory")?;
    let path = blobs_dir.join(hash);

    // Write to temp file then rename for atomic updates
    let temp_path = path.with_extension("tmp");
    let mut file = File::create(&temp_path)
        .with_context(|| format!("failed to create temp blob {}", temp_path.display()))?;

    bincode::serialize_into(&mut file, document).with_context(|| {
        format!(
            "failed to serialize document blob to {}",
            temp_path.display()
        )
    })?;

    fs::rename(&temp_path, &path)
        .with_context(|| format!("failed to atomically rename blob to {}", path.display()))?;

    Ok(())
}

pub fn load_blob(blobs_dir: &Path, hash: &str) -> Result<Document> {
    let path = blobs_dir.join(hash);
    let file =
        File::open(&path).with_context(|| format!("failed to open blob {}", path.display()))?;

    // Attempt mmap loading, fallback to standard I/O if it fails (e.g. empty file or OS restriction)
    let document: Document = match unsafe { memmap2::Mmap::map(&file) } {
        Ok(mmap) => bincode::deserialize(&mmap).with_context(|| {
            format!(
                "failed to deserialize document blob from mmap {}",
                path.display()
            )
        })?,
        Err(e) => {
            tracing::warn!(
                "failed to mmap blob {}, falling back to standard I/O: {}",
                path.display(),
                e
            );
            bincode::deserialize_from(&file).with_context(|| {
                format!(
                    "failed to deserialize document blob from file {}",
                    path.display()
                )
            })?
        }
    };

    Ok(document)
}
