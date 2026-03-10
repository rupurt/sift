pub mod model;
pub mod store;

pub use model::*;
pub use store::*;

use anyhow::{Result, bail};
use directories::ProjectDirs;
use std::env;
use std::path::PathBuf;

pub fn cache_dir(cache_type: &str) -> Result<PathBuf> {
    let env_var = format!("SIFT_{}_CACHE", cache_type.to_uppercase());
    if let Some(path) = env::var_os(env_var) {
        return Ok(PathBuf::from(path));
    }

    // Backward compatibility for SIFT_MODEL_CACHE
    if cache_type == "models" && let Some(path) = env::var_os("SIFT_MODEL_CACHE") {
        return Ok(PathBuf::from(path));
    }

    if let Some(path) = env::var_os("SIFT_CACHE") {
        return Ok(PathBuf::from(path).join(cache_type));
    }

    if let Some(proj_dirs) = ProjectDirs::from("com", "rupurt", "sift") {
        return Ok(proj_dirs.cache_dir().join(cache_type));
    }

    // Fallback for systems where ProjectDirs might fail
    if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(path).join("sift").join(cache_type));
    }
    if let Some(path) = env::var_os("HOME") {
        return Ok(PathBuf::from(path)
            .join(".cache")
            .join("sift")
            .join(cache_type));
    }

    bail!(
        "unable to determine a {} cache directory; set SIFT_{}_CACHE, SIFT_CACHE, XDG_CACHE_HOME, or HOME",
        cache_type,
        cache_type.to_uppercase()
    )
}
