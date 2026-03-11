use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "src/versioning.rs"]
mod versioning;

fn main() {
    println!("cargo:rerun-if-env-changed=SIFT_GIT_SHA");
    println!("cargo:rerun-if-env-changed=SIFT_RELEASE_BUILD");
    println!("cargo:rerun-if-changed=src/versioning.rs");

    emit_git_rerun_hints();

    let package_version = env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION is set by Cargo");
    let profile = env::var("PROFILE").unwrap_or_default();
    let release_override = env::var("SIFT_RELEASE_BUILD").ok();
    let git_sha = resolve_git_sha();
    let version = versioning::compose_version_string(
        &package_version,
        git_sha.as_deref(),
        versioning::is_release_build(&profile, release_override.as_deref()),
    );

    println!("cargo:rustc-env=SIFT_CLI_VERSION={version}");
}

fn resolve_git_sha() -> Option<String> {
    env::var("SIFT_GIT_SHA")
        .ok()
        .and_then(|value| versioning::normalize_sha(&value))
        .or_else(resolve_git_sha_from_repo)
}

fn resolve_git_sha_from_repo() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let sha = String::from_utf8(output.stdout).ok()?;
    versioning::normalize_sha(&sha)
}

fn emit_git_rerun_hints() {
    let Some(git_dir) = resolve_git_dir(Path::new(".")) else {
        return;
    };

    let head = git_dir.join("HEAD");
    println!("cargo:rerun-if-changed={}", head.display());

    let packed_refs = git_dir.join("packed-refs");
    println!("cargo:rerun-if-changed={}", packed_refs.display());

    if let Ok(head_contents) = fs::read_to_string(&head)
        && let Some(reference) = head_contents.strip_prefix("ref: ")
    {
        let ref_path = git_dir.join(reference.trim());
        println!("cargo:rerun-if-changed={}", ref_path.display());
    }
}

fn resolve_git_dir(repo_root: &Path) -> Option<PathBuf> {
    let dot_git = repo_root.join(".git");

    if dot_git.is_dir() {
        return Some(dot_git);
    }

    let git_pointer = fs::read_to_string(dot_git).ok()?;
    let path = git_pointer.trim().strip_prefix("gitdir: ")?;
    Some(repo_root.join(path))
}
