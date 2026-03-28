pub mod model;
pub mod store;

pub use model::*;
pub use store::*;

use anyhow::{Result, bail};
use directories::ProjectDirs;
use std::env;
use std::path::{Path, PathBuf};

pub fn cache_dir(cache_type: &str) -> Result<PathBuf> {
    let env_var = format!("SIFT_{}_CACHE", cache_type.to_uppercase());
    if let Some(path) = env::var_os(env_var) {
        return Ok(PathBuf::from(path));
    }

    // Backward compatibility for SIFT_MODEL_CACHE
    if cache_type == "models"
        && let Some(path) = env::var_os("SIFT_MODEL_CACHE")
    {
        return Ok(PathBuf::from(path));
    }

    if let Some(path) = env::var_os("SIFT_CACHE") {
        return Ok(PathBuf::from(path).join(cache_type));
    }

    #[cfg(unix)]
    if let Some(path) = unix_cache_root() {
        return Ok(path.join(cache_type));
    }

    if let Some(proj_dirs) = ProjectDirs::from("com", "rupurt", "sift") {
        return Ok(proj_dirs.cache_dir().join(cache_type));
    }

    #[cfg(not(unix))]
    if let Some(path) = unix_cache_root() {
        return Ok(path.join(cache_type));
    }

    bail!(
        "unable to determine a {} cache directory; set SIFT_{}_CACHE, SIFT_CACHE, XDG_CACHE_HOME, or HOME",
        cache_type,
        cache_type.to_uppercase()
    )
}

pub fn resolve_compatible_cache_path(path: &Path) -> PathBuf {
    resolve_compatible_cache_path_with_roots(
        path,
        unix_cache_root().as_deref(),
        project_cache_root().as_deref(),
    )
}

fn resolve_compatible_cache_path_with_roots(
    path: &Path,
    unix_root: Option<&Path>,
    project_root: Option<&Path>,
) -> PathBuf {
    if path.exists() {
        return path.to_path_buf();
    }

    let Some(unix_root) = unix_root else {
        return path.to_path_buf();
    };
    let Some(project_root) = project_root else {
        return path.to_path_buf();
    };

    if unix_root == project_root {
        return path.to_path_buf();
    }

    let translated = if let Ok(relative) = path.strip_prefix(unix_root) {
        project_root.join(relative)
    } else if let Ok(relative) = path.strip_prefix(project_root) {
        unix_root.join(relative)
    } else {
        return path.to_path_buf();
    };

    if translated.exists() {
        tracing::info!(
            requested = %path.display(),
            resolved = %translated.display(),
            "remapping compatible sift cache path"
        );
        return translated;
    }

    path.to_path_buf()
}

fn unix_cache_root() -> Option<PathBuf> {
    if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        return Some(PathBuf::from(path).join("sift"));
    }
    env::var_os("HOME").map(|path| PathBuf::from(path).join(".cache").join("sift"))
}

fn project_cache_root() -> Option<PathBuf> {
    ProjectDirs::from("com", "rupurt", "sift").map(|proj_dirs| proj_dirs.cache_dir().to_path_buf())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::resolve_compatible_cache_path_with_roots;

    #[test]
    fn remaps_unix_paths_when_project_cache_contains_the_target() {
        let root = tempdir().expect("temp root");
        let unix_root = root.path().join(".cache").join("sift");
        let project_root = root
            .path()
            .join("Library")
            .join("Caches")
            .join("com.rupurt.sift");
        let translated = project_root
            .join("eval")
            .join("scifact-files")
            .join("doc.txt");
        std::fs::create_dir_all(translated.parent().expect("translated parent"))
            .expect("create translated parent");
        std::fs::write(&translated, "document").expect("write translated doc");

        let unix_path = unix_root.join("eval").join("scifact-files").join("doc.txt");
        let resolved = resolve_compatible_cache_path_with_roots(
            &unix_path,
            Some(&unix_root),
            Some(&project_root),
        );

        assert_eq!(resolved, translated);
    }

    #[test]
    fn remaps_project_paths_when_unix_cache_contains_the_target() {
        let root = tempdir().expect("temp root");
        let unix_root = root.path().join(".cache").join("sift");
        let project_root = root
            .path()
            .join("Library")
            .join("Caches")
            .join("com.rupurt.sift");
        let translated = unix_root.join("eval").join("scifact-files").join("doc.txt");
        std::fs::create_dir_all(translated.parent().expect("translated parent"))
            .expect("create translated parent");
        std::fs::write(&translated, "document").expect("write translated doc");

        let resolved = resolve_compatible_cache_path_with_roots(
            &project_root
                .join("eval")
                .join("scifact-files")
                .join("doc.txt"),
            Some(&unix_root),
            Some(&project_root),
        );

        assert_eq!(resolved, translated);
    }

    #[test]
    fn keeps_existing_unix_paths_unchanged() {
        let root = tempdir().expect("temp root");
        let unix_root = root.path().join(".cache").join("sift");
        let project_root = root
            .path()
            .join("Library")
            .join("Caches")
            .join("com.rupurt.sift");
        let unix_path = unix_root.join("eval").join("scifact-files").join("doc.txt");
        std::fs::create_dir_all(unix_path.parent().expect("unix parent"))
            .expect("create unix parent");
        std::fs::write(&unix_path, "document").expect("write unix doc");

        let resolved = resolve_compatible_cache_path_with_roots(
            &unix_path,
            Some(&unix_root),
            Some(&project_root),
        );

        assert_eq!(resolved, unix_path);
    }
}
