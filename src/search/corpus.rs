use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;

use crate::cache::{Manifest, get_file_heuristics, hash_file, load_blob, save_blob};

use super::domain::{
    AcquisitionAdapterKind, AgentTurnInput, ArtifactBudget, ArtifactFreshness, ArtifactProvenance,
    ContextArtifact, ContextArtifactKind, Embedder, EnvironmentFactInput, Ignore, LoadedCorpus,
    LocalContextSource, ToolOutputInput,
};

const PROJECT_DOCS: &[&str] = &[
    "AGENTS.md",
    "INSTRUCTIONS.md",
    "WORLD.md",
    "CONSTITUTION.md",
    "ARCHITECTURE.md",
    "CONFIGURATION.md",
    "EVALUATIONS.md",
    "RESEARCH.md",
    "README.md",
    "RELEASE.md",
    "flake.nix",
];

pub fn load_search_corpus(
    root: &Path,
    ignore: Option<&Ignore>,
    _verbose: u8,
    _dense: Option<&dyn Embedder>,
    telemetry: &crate::system::Telemetry,
    local_context: &[LocalContextSource],
    cache_base: Option<&Path>,
) -> Result<LoadedCorpus> {
    if !root.exists() {
        bail!("search path '{}' does not exist", root.display());
    }

    let file_paths = collect_file_paths(root, ignore);
    let total_sources = file_paths.len() + local_context.len();
    telemetry
        .total_files
        .store(total_sources, Ordering::Relaxed);

    let cache_paths = cache_base.map(|base| CachePaths::for_root(base, root));
    let mut manifest = if let Some(paths) = &cache_paths {
        Manifest::load(&paths.manifest_path)?
    } else {
        Manifest::default()
    };

    let mut artifacts = Vec::new();

    for path in file_paths {
        match load_file_artifact(root, &path, &mut manifest, cache_paths.as_ref(), telemetry) {
            Ok(Some(artifact)) => {
                telemetry
                    .total_segments
                    .fetch_add(artifact.segments.len(), Ordering::Relaxed);
                artifacts.push(artifact);
            }
            Ok(None) => {}
            Err(error) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %error,
                    "skipping unreadable artifact during corpus load"
                );
            }
        }
    }

    for source in local_context {
        match load_local_context_artifact(source, cache_paths.as_ref(), telemetry) {
            Ok(Some(artifact)) => {
                telemetry
                    .total_segments
                    .fetch_add(artifact.segments.len(), Ordering::Relaxed);
                artifacts.push(artifact);
            }
            Ok(None) => {}
            Err(error) => {
                tracing::warn!(error = %error, "skipping invalid local context source");
            }
        }
    }

    if let Some(paths) = &cache_paths {
        manifest.save(&paths.manifest_path)?;
    }

    let total_bytes = artifacts
        .iter()
        .map(|artifact| artifact.length as u64)
        .sum();
    let indexed_artifacts = artifacts.len();
    let skipped_artifacts = total_sources.saturating_sub(indexed_artifacts);

    Ok(LoadedCorpus {
        artifacts,
        total_bytes,
        indexed_artifacts,
        skipped_artifacts,
    })
}

fn collect_file_paths(root: &Path, ignore: Option<&Ignore>) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if root.is_file() {
        if ignore.is_none_or(|ignore| !ignore.is_ignored(root)) {
            files.push(root.to_path_buf());
        }
        return files;
    }

    for entry in WalkDir::new(root).sort_by_file_name().into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if ignore.is_some_and(|ignore| ignore.is_ignored(path)) {
            continue;
        }
        files.push(path.to_path_buf());
    }

    files
}

fn load_file_artifact(
    root: &Path,
    path: &Path,
    manifest: &mut Manifest,
    cache_paths: Option<&CachePaths>,
    telemetry: &crate::system::Telemetry,
) -> Result<Option<ContextArtifact>> {
    if let Some(paths) = cache_paths
        && let Ok(current) = get_file_heuristics(path)
        && let Some(hash) = manifest.check_heuristics(path, &current)
    {
        telemetry.heuristic_hits.fetch_add(1, Ordering::Relaxed);
        if let Ok(artifact) = load_blob(&paths.blobs_dir, &hash) {
            telemetry.blob_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(Some(artifact));
        }
    }

    let extracted = match crate::extract::extract_path(path) {
        Ok(Some(artifact)) => artifact,
        Ok(None) => return Ok(None),
        Err(error) => return Err(error),
    };

    let content_hash = hash_file(path).with_context(|| format!("hash file {}", path.display()))?;
    let kind = classify_path_kind(root, path);
    let artifact = build_artifact(
        kind,
        path.to_path_buf(),
        extracted.source_kind,
        extracted.text,
        ArtifactProvenance {
            adapter: match kind {
                ContextArtifactKind::ProjectDocument => AcquisitionAdapterKind::ProjectDocument,
                _ => AcquisitionAdapterKind::FileSystem,
            },
            source: path.display().to_string(),
            synthetic: false,
        },
        ArtifactFreshness {
            observed_unix_secs: current_unix_secs(),
            modified_unix_secs: file_modified_unix_secs(path),
        },
        &content_hash,
    );

    if let Some(paths) = cache_paths {
        save_blob(&paths.blobs_dir, &content_hash, &artifact)?;
        if let Ok(current) = get_file_heuristics(path) {
            manifest.update(path.to_path_buf(), current, content_hash);
        }
    }

    Ok(Some(artifact))
}

fn load_local_context_artifact(
    source: &LocalContextSource,
    cache_paths: Option<&CachePaths>,
    telemetry: &crate::system::Telemetry,
) -> Result<Option<ContextArtifact>> {
    let observed_unix_secs = current_unix_secs();
    let synthetic = normalize_local_context(source, observed_unix_secs);
    let cache_key = synthetic_cache_key(&synthetic);

    if let Some(paths) = cache_paths
        && let Ok(artifact) = load_blob(&paths.blobs_dir, &cache_key)
    {
        telemetry.blob_hits.fetch_add(1, Ordering::Relaxed);
        return Ok(Some(artifact));
    }

    if let Some(paths) = cache_paths {
        save_blob(&paths.blobs_dir, &cache_key, &synthetic)?;
    }

    Ok(Some(synthetic))
}

fn normalize_local_context(
    source: &LocalContextSource,
    observed_unix_secs: i64,
) -> ContextArtifact {
    match source {
        LocalContextSource::EnvironmentFact(input) => {
            build_environment_artifact(input, observed_unix_secs)
        }
        LocalContextSource::ToolOutput(input) => {
            build_tool_output_artifact(input, observed_unix_secs)
        }
        LocalContextSource::AgentTurn(input) => {
            build_agent_turn_artifact(input, observed_unix_secs)
        }
    }
}

fn build_environment_artifact(
    input: &EnvironmentFactInput,
    observed_unix_secs: i64,
) -> ContextArtifact {
    let path = synthetic_path("environment", &input.key);
    let text = format!("{}={}", input.key, input.value);
    let source = format!("environment:{}", input.key);
    build_artifact(
        ContextArtifactKind::EnvironmentFact,
        path,
        crate::extract::SourceKind::Text,
        text,
        ArtifactProvenance {
            adapter: AcquisitionAdapterKind::EnvironmentContext,
            source,
            synthetic: true,
        },
        ArtifactFreshness {
            observed_unix_secs,
            modified_unix_secs: None,
        },
        &blake3::hash(input.value.as_bytes()).to_string(),
    )
}

fn build_tool_output_artifact(input: &ToolOutputInput, observed_unix_secs: i64) -> ContextArtifact {
    let path = synthetic_path(
        "tool-output",
        &format!("{}-{}", input.tool_name, input.call_id),
    );
    let source = format!("tool-output:{}:{}", input.tool_name, input.call_id);
    build_artifact(
        ContextArtifactKind::ToolOutput,
        path,
        crate::extract::SourceKind::Text,
        input.content.clone(),
        ArtifactProvenance {
            adapter: AcquisitionAdapterKind::ToolOutput,
            source,
            synthetic: true,
        },
        ArtifactFreshness {
            observed_unix_secs,
            modified_unix_secs: None,
        },
        &blake3::hash(input.content.as_bytes()).to_string(),
    )
}

fn build_agent_turn_artifact(input: &AgentTurnInput, observed_unix_secs: i64) -> ContextArtifact {
    let session = input.session_id.as_deref().unwrap_or("standalone");
    let path = synthetic_path("agent-turn", &format!("{session}-{}", input.turn_id));
    let text = format!("role: {}\n\n{}", input.role, input.content);
    let source = format!("agent-turn:{}:{}", session, input.turn_id);
    build_artifact(
        ContextArtifactKind::AgentTurn,
        path,
        crate::extract::SourceKind::Text,
        text,
        ArtifactProvenance {
            adapter: AcquisitionAdapterKind::AgentTurn,
            source,
            synthetic: true,
        },
        ArtifactFreshness {
            observed_unix_secs,
            modified_unix_secs: None,
        },
        &blake3::hash(input.content.as_bytes()).to_string(),
    )
}

fn build_artifact(
    kind: ContextArtifactKind,
    path: PathBuf,
    source_kind: crate::extract::SourceKind,
    text: String,
    provenance: ArtifactProvenance,
    freshness: ArtifactFreshness,
    content_hash: &str,
) -> ContextArtifact {
    let artifact_id = format!("{}:{}", path.display(), content_hash);
    let segments = crate::segment::build_segments(&artifact_id, &path, source_kind, &text);
    let terms =
        crate::search::domain::tokenize(&text)
            .into_iter()
            .fold(HashMap::new(), |mut acc, term| {
                *acc.entry(term).or_insert(0) += 1;
                acc
            });
    let budget = ArtifactBudget::from_text(&text, segments.len());

    ContextArtifact {
        id: artifact_id,
        kind,
        path,
        source_kind,
        length: text.len(),
        terms,
        text,
        segments,
        provenance,
        freshness,
        budget,
    }
}

fn classify_path_kind(root: &Path, path: &Path) -> ContextArtifactKind {
    let relative = if root.is_file() {
        path.file_name().and_then(|name| name.to_str())
    } else {
        path.strip_prefix(root).ok().and_then(|relative| {
            if relative.components().count() == 1 {
                relative.file_name().and_then(|name| name.to_str())
            } else {
                None
            }
        })
    };

    if relative.is_some_and(|name| PROJECT_DOCS.contains(&name)) {
        ContextArtifactKind::ProjectDocument
    } else {
        ContextArtifactKind::File
    }
}

fn synthetic_path(namespace: &str, key: &str) -> PathBuf {
    PathBuf::from(".sift")
        .join("context")
        .join(namespace)
        .join(format!("{}.txt", sanitize_component(key)))
}

fn sanitize_component(component: &str) -> String {
    let mut sanitized = String::with_capacity(component.len());
    for ch in component.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }
    sanitized.trim_matches('_').to_string()
}

fn synthetic_cache_key(artifact: &ContextArtifact) -> String {
    let material = format!(
        "{}\n{}\n{}\n{}",
        artifact.kind as u8,
        artifact.provenance.source,
        artifact.path.display(),
        artifact.text
    );
    blake3::hash(material.as_bytes()).to_string()
}

fn current_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

fn file_modified_unix_secs(path: &Path) -> Option<i64> {
    std::fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs() as i64)
}

struct CachePaths {
    manifest_path: PathBuf,
    blobs_dir: PathBuf,
}

impl CachePaths {
    fn for_root(base: &Path, root: &Path) -> Self {
        let corpus_key = blake3::hash(root.display().to_string().as_bytes()).to_string();
        Self {
            manifest_path: base.join("artifacts").join("manifests").join(corpus_key),
            blobs_dir: base.join("artifacts").join("blobs"),
        }
    }
}
