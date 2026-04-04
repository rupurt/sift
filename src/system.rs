use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Serialize, Deserialize)]
pub struct Telemetry {
    pub heuristic_hits: AtomicUsize,
    pub blob_hits: AtomicUsize,
    pub fresh_artifact_builds: AtomicUsize,
    pub skipped_artifacts: AtomicUsize,
    pub embedding_hits: AtomicUsize,
    pub total_files: AtomicUsize,
    pub total_segments: AtomicUsize,
    pub bm25_index_cache_hits: AtomicUsize,
    pub bm25_index_builds: AtomicUsize,
    pub sector_cache_hits: AtomicUsize,
    pub sector_rebuilds: AtomicUsize,
    pub sector_shard_cache_hits: AtomicUsize,
    pub sector_shard_builds: AtomicUsize,
}

impl Default for Telemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            heuristic_hits: AtomicUsize::new(0),
            blob_hits: AtomicUsize::new(0),
            fresh_artifact_builds: AtomicUsize::new(0),
            skipped_artifacts: AtomicUsize::new(0),
            embedding_hits: AtomicUsize::new(0),
            total_files: AtomicUsize::new(0),
            total_segments: AtomicUsize::new(0),
            bm25_index_cache_hits: AtomicUsize::new(0),
            bm25_index_builds: AtomicUsize::new(0),
            sector_cache_hits: AtomicUsize::new(0),
            sector_rebuilds: AtomicUsize::new(0),
            sector_shard_cache_hits: AtomicUsize::new(0),
            sector_shard_builds: AtomicUsize::new(0),
        }
    }

    pub fn reset(&self) {
        self.heuristic_hits.store(0, Ordering::Relaxed);
        self.blob_hits.store(0, Ordering::Relaxed);
        self.fresh_artifact_builds.store(0, Ordering::Relaxed);
        self.skipped_artifacts.store(0, Ordering::Relaxed);
        self.embedding_hits.store(0, Ordering::Relaxed);
        self.total_files.store(0, Ordering::Relaxed);
        self.total_segments.store(0, Ordering::Relaxed);
        self.bm25_index_cache_hits.store(0, Ordering::Relaxed);
        self.bm25_index_builds.store(0, Ordering::Relaxed);
        self.sector_cache_hits.store(0, Ordering::Relaxed);
        self.sector_rebuilds.store(0, Ordering::Relaxed);
        self.sector_shard_cache_hits.store(0, Ordering::Relaxed);
        self.sector_shard_builds.store(0, Ordering::Relaxed);
    }

    pub fn heuristic_hit_rate(&self) -> f64 {
        let total = self.total_files.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            self.heuristic_hits.load(Ordering::Relaxed) as f64 / total as f64
        }
    }

    pub fn blob_hit_rate(&self) -> f64 {
        let total = self.total_files.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            self.blob_hits.load(Ordering::Relaxed) as f64 / total as f64
        }
    }

    pub fn embedding_hit_rate(&self) -> f64 {
        let total = self.total_segments.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            self.embedding_hits.load(Ordering::Relaxed) as f64 / total as f64
        }
    }
}

impl Clone for Telemetry {
    fn clone(&self) -> Self {
        Self {
            heuristic_hits: AtomicUsize::new(self.heuristic_hits.load(Ordering::Relaxed)),
            blob_hits: AtomicUsize::new(self.blob_hits.load(Ordering::Relaxed)),
            fresh_artifact_builds: AtomicUsize::new(
                self.fresh_artifact_builds.load(Ordering::Relaxed),
            ),
            skipped_artifacts: AtomicUsize::new(self.skipped_artifacts.load(Ordering::Relaxed)),
            embedding_hits: AtomicUsize::new(self.embedding_hits.load(Ordering::Relaxed)),
            total_files: AtomicUsize::new(self.total_files.load(Ordering::Relaxed)),
            total_segments: AtomicUsize::new(self.total_segments.load(Ordering::Relaxed)),
            bm25_index_cache_hits: AtomicUsize::new(
                self.bm25_index_cache_hits.load(Ordering::Relaxed),
            ),
            bm25_index_builds: AtomicUsize::new(self.bm25_index_builds.load(Ordering::Relaxed)),
            sector_cache_hits: AtomicUsize::new(self.sector_cache_hits.load(Ordering::Relaxed)),
            sector_rebuilds: AtomicUsize::new(self.sector_rebuilds.load(Ordering::Relaxed)),
            sector_shard_cache_hits: AtomicUsize::new(
                self.sector_shard_cache_hits.load(Ordering::Relaxed),
            ),
            sector_shard_builds: AtomicUsize::new(self.sector_shard_builds.load(Ordering::Relaxed)),
        }
    }
}

impl PartialEq for Telemetry {
    fn eq(&self, other: &Self) -> bool {
        self.heuristic_hits.load(Ordering::Relaxed) == other.heuristic_hits.load(Ordering::Relaxed)
            && self.blob_hits.load(Ordering::Relaxed) == other.blob_hits.load(Ordering::Relaxed)
            && self.fresh_artifact_builds.load(Ordering::Relaxed)
                == other.fresh_artifact_builds.load(Ordering::Relaxed)
            && self.skipped_artifacts.load(Ordering::Relaxed)
                == other.skipped_artifacts.load(Ordering::Relaxed)
            && self.embedding_hits.load(Ordering::Relaxed)
                == other.embedding_hits.load(Ordering::Relaxed)
            && self.total_files.load(Ordering::Relaxed) == other.total_files.load(Ordering::Relaxed)
            && self.total_segments.load(Ordering::Relaxed)
                == other.total_segments.load(Ordering::Relaxed)
            && self.bm25_index_cache_hits.load(Ordering::Relaxed)
                == other.bm25_index_cache_hits.load(Ordering::Relaxed)
            && self.bm25_index_builds.load(Ordering::Relaxed)
                == other.bm25_index_builds.load(Ordering::Relaxed)
            && self.sector_cache_hits.load(Ordering::Relaxed)
                == other.sector_cache_hits.load(Ordering::Relaxed)
            && self.sector_rebuilds.load(Ordering::Relaxed)
                == other.sector_rebuilds.load(Ordering::Relaxed)
            && self.sector_shard_cache_hits.load(Ordering::Relaxed)
                == other.sector_shard_cache_hits.load(Ordering::Relaxed)
            && self.sector_shard_builds.load(Ordering::Relaxed)
                == other.sector_shard_builds.load(Ordering::Relaxed)
    }
}

pub fn current_git_sha() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareSummary {
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub total_memory_gb: u64,
    pub os: String,
    pub arch: String,
}

pub fn detect_hardware_summary() -> HardwareSummary {
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_all();

    HardwareSummary {
        cpu_brand: sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        cpu_cores: sys.cpus().len(),
        total_memory_gb: sys.total_memory() / 1024 / 1024 / 1024,
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    }
}
