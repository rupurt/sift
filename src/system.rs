use serde::{Deserialize, Serialize};
use std::process::Command;
use sysinfo::System;

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
