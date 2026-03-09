use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use sysinfo::System;

#[derive(Debug, Default)]
pub struct Telemetry {
    pub heuristic_hits: AtomicUsize,
    pub blob_hits: AtomicUsize,
    pub embedding_hits: AtomicUsize,
    pub total_files: AtomicUsize,
    pub total_segments: AtomicUsize,
}

impl Telemetry {
    pub fn new() -> Self {
        Self::default()
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

    pub fn trace_hit_rates(&self) {
        tracing::info!(
            "cache hit rates: heuristic={:.1}%, blob={:.1}%, embedding={:.1}%",
            self.heuristic_hit_rate() * 100.0,
            self.blob_hit_rate() * 100.0,
            self.embedding_hit_rate() * 100.0
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareSummary {
    pub os: String,
    pub arch: String,
    pub logical_cores: usize,
    pub cpu_brand: Option<String>,
    pub total_memory_bytes: Option<u64>,
}

pub fn detect_hardware_summary() -> HardwareSummary {
    let mut system = System::new_all();
    system.refresh_all();

    let cpu_brand = system
        .cpus()
        .first()
        .map(|cpu| cpu.brand().trim().to_string())
        .filter(|brand| !brand.is_empty());

    HardwareSummary {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        logical_cores: std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1),
        cpu_brand,
        total_memory_bytes: Some(system.total_memory()),
    }
}

pub fn current_git_sha() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let sha = String::from_utf8(output.stdout).ok()?;
    let sha = sha.trim();
    if sha.is_empty() {
        None
    } else {
        Some(sha.to_string())
    }
}
