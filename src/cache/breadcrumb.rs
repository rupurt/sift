use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::cache::{corpus_cache_key, load_locked_bincode, save_locked_bincode};

pub const BREADCRUMB_SCHEMA_VERSION: u32 = 1;
pub const BREADCRUMB_STALE_AFTER_SECS: i64 = 12 * 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BreadcrumbJournal {
    pub schema_version: u32,
    pub corpus_key: String,
    pub run_id: String,
    pub updated_at_unix_secs: i64,
    pub dirty_sectors: Vec<String>,
    pub completed_sectors: Vec<String>,
    pub active_sector: Option<ActiveBreadcrumbSector>,
}

impl BreadcrumbJournal {
    pub fn new(corpus_key: String, run_id: String, dirty_sectors: Vec<String>) -> Self {
        Self {
            schema_version: BREADCRUMB_SCHEMA_VERSION,
            corpus_key,
            run_id,
            updated_at_unix_secs: 0,
            dirty_sectors,
            completed_sectors: Vec::new(),
            active_sector: None,
        }
    }

    pub fn load(path: &Path) -> Result<Option<Self>> {
        load_locked_bincode(path)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        save_locked_bincode(path, self)
    }

    pub fn load_for_root(cache_base: &Path, root: &Path) -> Result<Option<Self>> {
        Self::load(&breadcrumb_cache_path(cache_base, root))
    }

    pub fn save_for_root(&self, cache_base: &Path, root: &Path) -> Result<()> {
        self.save(&breadcrumb_cache_path(cache_base, root))
    }

    pub fn clear_for_root(cache_base: &Path, root: &Path) -> Result<()> {
        let path = breadcrumb_cache_path(cache_base, root);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    pub fn is_stale(&self, now_unix_secs: i64) -> bool {
        self.updated_at_unix_secs > 0
            && now_unix_secs.saturating_sub(self.updated_at_unix_secs) > BREADCRUMB_STALE_AFTER_SECS
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveBreadcrumbSector {
    pub sector_id: String,
    pub next_member_offset: usize,
    pub next_member_relative_path: Option<PathBuf>,
    pub sector_member_count: usize,
}

pub fn breadcrumb_cache_path(cache_base: &Path, root: &Path) -> PathBuf {
    cache_base
        .join("artifacts")
        .join("breadcrumbs")
        .join(format!("{}.bin", corpus_cache_key(root)))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempfile::tempdir;

    use super::{ActiveBreadcrumbSector, BreadcrumbJournal, breadcrumb_cache_path};

    #[test]
    fn breadcrumb_journal_round_trips_through_cache_storage() {
        let temp = tempdir().expect("tempdir");
        let cache_base = temp.path().join("cache");
        let root = Path::new("/repo");
        let journal = BreadcrumbJournal {
            schema_version: 1,
            corpus_key: "corpus".to_string(),
            run_id: "run-1".to_string(),
            updated_at_unix_secs: 123,
            dirty_sectors: vec!["sector-a".to_string(), "sector-b".to_string()],
            completed_sectors: vec!["sector-a".to_string()],
            active_sector: Some(ActiveBreadcrumbSector {
                sector_id: "sector-b".to_string(),
                next_member_offset: 1,
                next_member_relative_path: Some("src/lib.rs".into()),
                sector_member_count: 2,
            }),
        };

        journal
            .save_for_root(&cache_base, root)
            .expect("save breadcrumb");
        let loaded = BreadcrumbJournal::load_for_root(&cache_base, root)
            .expect("load breadcrumb")
            .expect("breadcrumb should exist");

        assert_eq!(loaded, journal);
    }

    #[test]
    fn breadcrumb_cache_path_uses_existing_corpus_key_namespace() {
        let path = breadcrumb_cache_path(Path::new("/tmp/sift-cache"), Path::new("/repo"));
        assert!(path.starts_with("/tmp/sift-cache/artifacts/breadcrumbs"));
    }
}
