use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use fs2::FileExt;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::cache::model::{CacheEntry, Manifest};
use crate::search::domain::ContextArtifact;

pub fn hash_file(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut hasher = blake3::Hasher::new();
    std::io::copy(&mut file, &mut hasher)
        .with_context(|| format!("failed to hash {}", path.display()))?;
    Ok(hasher.finalize().to_string())
}

pub fn get_file_heuristics(path: &Path) -> Result<CacheEntry> {
    let meta = fs::metadata(path).with_context(|| format!("failed to stat {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        Ok(CacheEntry {
            inode: meta.ino(),
            mtime_secs: meta.mtime(),
            mtime_nanos: meta.mtime_nsec() as u32,
            size: meta.size(),
            blake3_hash: String::new(), // Will be populated if needed
        })
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;

        let last_write_time = meta.last_write_time();
        Ok(CacheEntry {
            // Use creation time as the closest stable file identity available in std on Windows.
            inode: meta.creation_time(),
            mtime_secs: (last_write_time / 10_000_000) as i64,
            mtime_nanos: ((last_write_time % 10_000_000) * 100) as u32,
            size: meta.file_size(),
            blake3_hash: String::new(), // Will be populated if needed
        })
    }

    #[cfg(not(any(unix, windows)))]
    {
        let modified = meta
            .modified()
            .with_context(|| format!("failed to read mtime for {}", path.display()))?;
        let since_epoch = modified
            .duration_since(std::time::UNIX_EPOCH)
            .with_context(|| format!("mtime predates UNIX_EPOCH for {}", path.display()))?;

        Ok(CacheEntry {
            inode: 0,
            mtime_secs: since_epoch.as_secs() as i64,
            mtime_nanos: since_epoch.subsec_nanos(),
            size: meta.len(),
            blake3_hash: String::new(), // Will be populated if needed
        })
    }
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self> {
        load_locked_bincode_default(path)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        save_locked_bincode(path, self)
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

pub(crate) fn load_locked_bincode_default<T>(path: &Path) -> Result<T>
where
    T: Default + DeserializeOwned,
{
    if !path.exists() {
        return Ok(T::default());
    }

    let file = File::open(path)
        .with_context(|| format!("failed to open cache file {}", path.display()))?;
    file.lock_shared()
        .with_context(|| format!("failed to acquire shared lock on {}", path.display()))?;

    let value = bincode::deserialize_from(&file).unwrap_or_default();

    file.unlock()
        .with_context(|| format!("failed to release shared lock on {}", path.display()))?;
    Ok(value)
}

pub(crate) fn save_locked_bincode<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create cache directory {}", parent.display()))?;
    }

    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .with_context(|| format!("failed to open cache file for writing {}", path.display()))?;

    file.lock_exclusive()
        .with_context(|| format!("failed to acquire exclusive lock on {}", path.display()))?;

    bincode::serialize_into(&file, value)
        .with_context(|| format!("failed to serialize cache file {}", path.display()))?;

    file.unlock()
        .with_context(|| format!("failed to release exclusive lock on {}", path.display()))?;
    Ok(())
}

pub fn save_blob(blobs_dir: &Path, hash: &str, document: &ContextArtifact) -> Result<()> {
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

pub fn load_blob(blobs_dir: &Path, hash: &str) -> Result<ContextArtifact> {
    let path = blobs_dir.join(hash);
    let file =
        File::open(&path).with_context(|| format!("failed to open blob {}", path.display()))?;

    // Attempt mmap loading, fallback to standard I/O if it fails (e.g. empty file or OS restriction)
    let document: ContextArtifact = match unsafe { memmap2::Mmap::map(&file) } {
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
