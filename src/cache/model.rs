use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Manifest {
    pub entries: HashMap<PathBuf, CacheEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    pub inode: u64,
    pub mtime_secs: i64,
    pub mtime_nanos: u32,
    pub size: u64,
    pub blake3_hash: String,
}
