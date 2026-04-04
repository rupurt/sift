use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;

use crate::cache::{
    ActiveBreadcrumbSector, BREADCRUMB_SCHEMA_VERSION, BreadcrumbJournal, CacheEntry,
    FrontierLedger, Manifest, SectorLexicalShardFormat, SectorLexicalShardRef, SectorMap,
    SectorMemberInput, SectorPartitionStrategy, corpus_cache_key, get_file_heuristics, hash_file,
    load_blob, load_locked_bincode, resolve_compatible_cache_path, resolve_sector_member_path,
    save_blob, save_locked_bincode, sector_bm25_shard_cache_path, sector_relative_path,
};
use blake3::Hasher;

use super::domain::{
    AcquisitionAdapterKind, AgentTurnInput, ArtifactBudget, ArtifactFreshness, ArtifactProvenance,
    Bm25Index, ContextArtifact, ContextArtifactKind, Embedder, EnvironmentFactInput, Ignore,
    LoadedCorpus, LocalContextSource, ToolOutputInput,
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
    load_search_corpus_with_progress(
        root,
        ignore,
        _verbose,
        _dense,
        telemetry,
        local_context,
        cache_base,
        None::<fn(&super::domain::SearchProgress)>,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn load_search_corpus_with_progress<F: Fn(&super::domain::SearchProgress)>(
    root: &Path,
    ignore: Option<&Ignore>,
    _verbose: u8,
    _dense: Option<&dyn Embedder>,
    telemetry: &crate::system::Telemetry,
    local_context: &[LocalContextSource],
    cache_base: Option<&Path>,
    progress: Option<F>,
) -> Result<LoadedCorpus> {
    let root = resolve_compatible_cache_path(root);
    if !root.exists() {
        bail!("search path '{}' does not exist", root.display());
    }

    let file_paths = collect_file_paths(&root, ignore);
    let total_sources = file_paths.len() + local_context.len();
    telemetry
        .total_files
        .store(total_sources, Ordering::Relaxed);

    let cache_paths = cache_base.map(|base| CachePaths::for_root(base, &root));
    let mut manifest = if let Some(paths) = &cache_paths {
        Manifest::load(&paths.manifest_path)?
    } else {
        Manifest::default()
    };

    let mut artifacts = Vec::new();
    let files_total = total_sources;
    let mut files_processed = 0usize;
    let indexing_started_at = Instant::now();
    let estimate_remaining = |processed: usize, total: usize| -> Option<Duration> {
        if processed == 0 || total == 0 {
            return None;
        }

        let elapsed = indexing_started_at.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            return None;
        }

        let remaining = total.saturating_sub(processed);
        if remaining == 0 {
            return Some(Duration::from_secs(0));
        }

        Some(Duration::from_secs_f64(
            elapsed * (remaining as f64 / processed as f64),
        ))
    };

    let mut emit_file_progress = || {
        files_processed += 1;
        if let Some(ref cb) = progress {
            cb(&super::domain::SearchProgress::Indexing {
                phase: super::domain::SearchPhase::Indexing,
                files_processed,
                files_total,
                estimated_remaining: estimate_remaining(files_processed, files_total),
                coverage: super::domain::SearchCoverageSnapshot::from_frontier(
                    &telemetry.frontier_snapshot(),
                ),
            });
        }
    };

    let mut file_artifacts = if let (Some(cache_base), Some(cache_paths)) =
        (cache_base, cache_paths.as_ref())
    {
        load_file_artifacts_with_sector_cache(
            &root,
            &file_paths,
            &mut manifest,
            cache_base,
            cache_paths,
            telemetry,
            &mut emit_file_progress,
        )?
    } else {
        let mut artifacts = Vec::new();
        for path in &file_paths {
            match load_file_artifact(&root, path, &mut manifest, cache_paths.as_ref(), telemetry) {
                Ok(Some(artifact)) => record_loaded_artifact(&mut artifacts, artifact, telemetry),
                Ok(None) => {
                    telemetry.skipped_artifacts.fetch_add(1, Ordering::Relaxed);
                }
                Err(error) => {
                    telemetry.skipped_artifacts.fetch_add(1, Ordering::Relaxed);
                    tracing::warn!(
                        path = %path.display(),
                        error = %error,
                        "skipping unreadable artifact during corpus load"
                    );
                }
            }
            emit_file_progress();
        }
        artifacts
    };
    artifacts.append(&mut file_artifacts);

    for source in local_context {
        match load_local_context_artifact(source, cache_paths.as_ref(), telemetry) {
            Ok(Some(artifact)) => {
                telemetry
                    .total_segments
                    .fetch_add(artifact.segments.len(), Ordering::Relaxed);
                artifacts.push(artifact);
            }
            Ok(None) => {
                telemetry.skipped_artifacts.fetch_add(1, Ordering::Relaxed);
            }
            Err(error) => {
                telemetry.skipped_artifacts.fetch_add(1, Ordering::Relaxed);
                tracing::warn!(error = %error, "skipping invalid local context source");
            }
        }
        files_processed += 1;
        if let Some(ref cb) = progress {
            cb(&super::domain::SearchProgress::Indexing {
                phase: super::domain::SearchPhase::Indexing,
                files_processed,
                files_total,
                estimated_remaining: estimate_remaining(files_processed, files_total),
                coverage: super::domain::SearchCoverageSnapshot::from_frontier(
                    &telemetry.frontier_snapshot(),
                ),
            });
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

pub fn compute_bm25_index_signature(artifacts: &[ContextArtifact]) -> String {
    let mut hasher = Hasher::new();
    for artifact in artifacts {
        hasher.update(artifact.id.as_bytes());
        hasher.update(&artifact.length.to_le_bytes());
    }
    hasher.finalize().to_string()
}

pub fn bm25_index_cache_path(cache_base: &Path, root: &Path, signature: &str) -> PathBuf {
    cache_base
        .join("artifacts")
        .join("indexes")
        .join(corpus_cache_key(root))
        .join(format!("{}.bin", signature))
}

pub fn load_bm25_index_cache(
    cache_base: &Path,
    root: &Path,
    signature: &str,
) -> Result<Option<crate::search::domain::Bm25Index>> {
    let path = bm25_index_cache_path(cache_base, root, signature);
    if !path.exists() {
        return Ok(None);
    }

    let file = File::open(&path)
        .with_context(|| format!("failed to open bm25 index cache {}", path.display()))?;
    let index = bincode::deserialize_from::<_, crate::search::domain::Bm25Index>(file)
        .with_context(|| format!("failed to deserialize bm25 index cache {}", path.display()))?;
    Ok(Some(index))
}

pub fn save_bm25_index_cache(
    cache_base: &Path,
    root: &Path,
    signature: &str,
    index: &crate::search::domain::Bm25Index,
) -> Result<()> {
    let path = bm25_index_cache_path(cache_base, root, signature);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("failed to create bm25 index cache directory")?;
    }

    let temp_path = path.with_extension("tmp");
    {
        let mut file = File::create(&temp_path).with_context(|| {
            format!(
                "failed to create temporary bm25 index cache {}",
                temp_path.display()
            )
        })?;
        bincode::serialize_into(&mut file, index).with_context(|| {
            format!(
                "failed to write bm25 index cache to {}",
                temp_path.display()
            )
        })?;
    }

    std::fs::rename(&temp_path, &path).with_context(|| {
        format!(
            "failed to atomically write bm25 index cache {}",
            path.display()
        )
    })?;

    Ok(())
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

#[derive(Clone)]
struct FileSectorCandidate {
    path: PathBuf,
    current: Option<CacheEntry>,
    cached_hash: Option<String>,
}

struct RebuildHooks<'a> {
    after_breadcrumb_checkpoint: Option<&'a mut dyn FnMut(&BreadcrumbJournal) -> Result<()>>,
}

impl<'a> Default for RebuildHooks<'a> {
    fn default() -> Self {
        Self {
            after_breadcrumb_checkpoint: None,
        }
    }
}

fn load_file_artifacts_with_sector_cache<F: FnMut()>(
    root: &Path,
    file_paths: &[PathBuf],
    manifest: &mut Manifest,
    cache_base: &Path,
    cache_paths: &CachePaths,
    telemetry: &crate::system::Telemetry,
    on_processed: &mut F,
) -> Result<Vec<ContextArtifact>> {
    let mut hooks = RebuildHooks::default();
    load_file_artifacts_with_sector_cache_impl(
        root,
        file_paths,
        manifest,
        cache_base,
        cache_paths,
        telemetry,
        on_processed,
        &mut hooks,
    )
}

fn load_file_artifacts_with_sector_cache_impl<F: FnMut()>(
    root: &Path,
    file_paths: &[PathBuf],
    manifest: &mut Manifest,
    cache_base: &Path,
    cache_paths: &CachePaths,
    telemetry: &crate::system::Telemetry,
    on_processed: &mut F,
    hooks: &mut RebuildHooks<'_>,
) -> Result<Vec<ContextArtifact>> {
    let cached_sector_map = match SectorMap::load_for_root(cache_base, root) {
        Ok(map) => map,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "failed to load sector map; falling back to empty sector state"
            );
            SectorMap::default()
        }
    };
    let mut tracked_inputs = Vec::new();
    let mut tracked_candidates = HashMap::new();

    for path in file_paths {
        let relative_path = sector_relative_path(root, path);
        let current = get_file_heuristics(path).ok();
        let cached_hash = current
            .as_ref()
            .and_then(|entry| manifest.check_heuristics(path, entry));
        let candidate = FileSectorCandidate {
            path: path.clone(),
            current,
            cached_hash,
        };

        tracked_inputs.push(SectorMemberInput::filesystem(
            candidate.path.clone(),
            candidate
                .cached_hash
                .clone()
                .unwrap_or_else(|| dirty_sector_placeholder(candidate.current.as_ref())),
        ));
        tracked_candidates.insert(relative_path, candidate);
    }

    let provisional_map =
        SectorMap::build_for_root(root, tracked_inputs, SectorPartitionStrategy::default())?;
    let cached_sectors_by_id = cached_sector_map
        .sectors
        .iter()
        .map(|sector| (sector.sector_id.clone(), sector))
        .collect::<HashMap<_, _>>();
    let mut file_artifacts = Vec::new();
    let mut clean_sector_ids = HashSet::new();
    let mut frontier =
        FrontierLedger::new(provisional_map.sectors.len(), provisional_map.sectors.len());
    telemetry.replace_frontier_ledger(frontier.clone());

    for sector in &provisional_map.sectors {
        let Some(cached_sector) = cached_sectors_by_id.get(&sector.sector_id) else {
            continue;
        };

        if cached_sector.membership.proof_fingerprint != sector.membership.proof_fingerprint
            || !sector_bm25_shard_available(cache_base, root, cached_sector)
        {
            continue;
        }

        match load_sector_artifacts_from_cache(&cache_paths.blobs_dir, cached_sector) {
            Ok(artifacts) => {
                clean_sector_ids.insert(sector.sector_id.clone());
                frontier.record_clean_mount(sector.sector_id.clone());
                telemetry.replace_frontier_ledger(frontier.clone());
                for artifact in artifacts {
                    record_loaded_artifact(&mut file_artifacts, artifact, telemetry);
                    on_processed();
                }
            }
            Err(error) => {
                tracing::warn!(
                    sector = %sector.sector_id,
                    error = %error,
                    "failed to load cached sector artifacts; rebuilding sector"
                );
            }
        }
    }

    let dirty_sector_ids = load_dirty_sector_ids(&provisional_map, &clean_sector_ids);
    let mut breadcrumb = load_resumable_breadcrumb_journal(
        root,
        cache_base,
        cache_paths,
        &provisional_map,
        &dirty_sector_ids,
        telemetry,
    )?;
    if breadcrumb.is_none() && !dirty_sector_ids.is_empty() {
        breadcrumb = Some(BreadcrumbJournal::new(
            provisional_map.corpus_key.clone(),
            generate_breadcrumb_run_id(),
            dirty_sector_ids.clone(),
        ));
    }
    let resumed_completed_sector_ids = breadcrumb
        .as_ref()
        .map(|journal| {
            journal
                .completed_sectors
                .iter()
                .cloned()
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    let resumed_active_sector = breadcrumb
        .as_ref()
        .and_then(|journal| journal.active_sector.clone());
    if let Some(journal) = breadcrumb.as_ref() {
        frontier.apply_breadcrumb_resume(journal);
        telemetry.replace_frontier_ledger(frontier.clone());
    }

    for sector in provisional_map
        .sectors
        .iter()
        .filter(|sector| !clean_sector_ids.contains(&sector.sector_id))
    {
        if resumed_completed_sector_ids.contains(&sector.sector_id) {
            for artifact in load_sector_artifacts_from_cache(&cache_paths.blobs_dir, sector)? {
                record_loaded_artifact(&mut file_artifacts, artifact, telemetry);
                on_processed();
            }
            continue;
        }

        let resume_offset = if resumed_active_sector
            .as_ref()
            .is_some_and(|active| active.sector_id == sector.sector_id)
        {
            let offset = resumed_active_sector
                .as_ref()
                .map(|active| active.next_member_offset)
                .unwrap_or_default();
            for artifact in
                load_sector_artifacts_from_cache_prefix(&cache_paths.blobs_dir, sector, offset)?
            {
                record_loaded_artifact(&mut file_artifacts, artifact, telemetry);
                on_processed();
            }
            offset
        } else {
            0
        };

        if let Some(journal) = breadcrumb.as_mut()
            && !resumed_active_sector
                .as_ref()
                .is_some_and(|active| active.sector_id == sector.sector_id)
        {
            frontier.start_dirty_rebuild(sector.sector_id.clone(), sector.proofs.len(), false, 0);
            telemetry.replace_frontier_ledger(frontier.clone());
            journal.active_sector = Some(ActiveBreadcrumbSector {
                sector_id: sector.sector_id.clone(),
                next_member_offset: resume_offset,
                next_member_relative_path: sector
                    .proofs
                    .get(resume_offset)
                    .map(|proof| proof.relative_path.clone()),
                sector_member_count: sector.proofs.len(),
            });
            checkpoint_breadcrumb(journal, cache_base, root, hooks)?;
        }

        for (member_index, proof) in sector.proofs.iter().enumerate().skip(resume_offset) {
            let path = tracked_candidates
                .get(&proof.relative_path)
                .map(|candidate| candidate.path.clone())
                .unwrap_or_else(|| resolve_sector_member_path(root, &proof.relative_path));
            load_file_artifact_with_tracking(
                root,
                &path,
                manifest,
                Some(cache_paths),
                telemetry,
                &mut file_artifacts,
            );
            persist_manifest_checkpoint(manifest, cache_paths)?;
            frontier.advance_dirty_rebuild(&sector.sector_id, member_index + 1);
            telemetry.replace_frontier_ledger(frontier.clone());
            if let Some(journal) = breadcrumb.as_mut() {
                if let Some(active) = journal.active_sector.as_mut() {
                    active.next_member_offset = member_index + 1;
                    active.next_member_relative_path = sector
                        .proofs
                        .get(member_index + 1)
                        .map(|next| next.relative_path.clone());
                }
                checkpoint_breadcrumb(journal, cache_base, root, hooks)?;
            }
            on_processed();
        }

        let sector_reused = resumed_active_sector.as_ref().is_some_and(|active| {
            active.sector_id == sector.sector_id && active.next_member_offset >= sector.proofs.len()
        });
        frontier.complete_dirty_rebuild(sector.sector_id.clone(), sector_reused);
        telemetry.replace_frontier_ledger(frontier.clone());
        if let Some(journal) = breadcrumb.as_mut() {
            journal.completed_sectors.push(sector.sector_id.clone());
            journal.active_sector = None;
            checkpoint_breadcrumb(journal, cache_base, root, hooks)?;
        }
    }

    if breadcrumb.is_some() {
        BreadcrumbJournal::clear_for_root(cache_base, root)?;
    }

    let order = file_paths
        .iter()
        .enumerate()
        .map(|(index, path)| (path.clone(), index))
        .collect::<HashMap<_, _>>();
    file_artifacts.sort_by_key(|artifact| order.get(&artifact.path).copied().unwrap_or(usize::MAX));

    persist_sector_map_and_shards(
        root,
        cache_base,
        &file_artifacts,
        &cached_sector_map,
        telemetry,
    )?;

    Ok(file_artifacts)
}

fn checkpoint_breadcrumb(
    journal: &mut BreadcrumbJournal,
    cache_base: &Path,
    root: &Path,
    hooks: &mut RebuildHooks<'_>,
) -> Result<()> {
    journal.updated_at_unix_secs = current_unix_secs();
    journal.save_for_root(cache_base, root)?;
    if let Some(callback) = hooks.after_breadcrumb_checkpoint.as_mut() {
        callback(journal)?;
    }
    Ok(())
}

fn generate_breadcrumb_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{}-{}-{nanos}", current_unix_secs(), std::process::id())
}

fn load_dirty_sector_ids(
    provisional_map: &SectorMap,
    clean_sector_ids: &HashSet<String>,
) -> Vec<String> {
    provisional_map
        .sectors
        .iter()
        .filter(|sector| !clean_sector_ids.contains(&sector.sector_id))
        .map(|sector| sector.sector_id.clone())
        .collect()
}

fn load_resumable_breadcrumb_journal(
    root: &Path,
    cache_base: &Path,
    cache_paths: &CachePaths,
    provisional_map: &SectorMap,
    dirty_sector_ids: &[String],
    telemetry: &crate::system::Telemetry,
) -> Result<Option<BreadcrumbJournal>> {
    let journal = match BreadcrumbJournal::load_for_root(cache_base, root) {
        Ok(journal) => journal,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "discarding unreadable breadcrumb journal before sector rebuild"
            );
            discard_breadcrumb_journal(cache_base, root, telemetry)?;
            return Ok(None);
        }
    };
    let Some(journal) = journal else {
        return Ok(None);
    };

    if let Err(reason) = validate_breadcrumb_journal(
        &journal,
        provisional_map,
        dirty_sector_ids,
        &cache_paths.blobs_dir,
    ) {
        tracing::warn!(
            reason,
            "discarding incompatible breadcrumb journal before sector rebuild"
        );
        discard_breadcrumb_journal(cache_base, root, telemetry)?;
        return Ok(None);
    }

    telemetry
        .breadcrumb_resume_hits
        .fetch_add(1, Ordering::Relaxed);
    Ok(Some(journal))
}

fn discard_breadcrumb_journal(
    cache_base: &Path,
    root: &Path,
    telemetry: &crate::system::Telemetry,
) -> Result<()> {
    telemetry
        .breadcrumb_recovery_discards
        .fetch_add(1, Ordering::Relaxed);
    BreadcrumbJournal::clear_for_root(cache_base, root)
}

fn validate_breadcrumb_journal(
    journal: &BreadcrumbJournal,
    provisional_map: &SectorMap,
    dirty_sector_ids: &[String],
    blobs_dir: &Path,
) -> std::result::Result<(), String> {
    if journal.schema_version != BREADCRUMB_SCHEMA_VERSION {
        return Err(format!(
            "schema version mismatch: {} != {}",
            journal.schema_version, BREADCRUMB_SCHEMA_VERSION
        ));
    }
    if journal.corpus_key != provisional_map.corpus_key {
        return Err("corpus key mismatch".to_string());
    }
    if journal.is_stale(current_unix_secs()) {
        return Err("breadcrumb journal is stale".to_string());
    }
    if journal.dirty_sectors != dirty_sector_ids {
        return Err("dirty sector set mismatch".to_string());
    }
    if journal.completed_sectors.len() > dirty_sector_ids.len() {
        return Err("completed sectors exceed dirty sector count".to_string());
    }
    if journal.completed_sectors.as_slice() != &dirty_sector_ids[..journal.completed_sectors.len()]
    {
        return Err("completed sectors do not match the dirty sector prefix".to_string());
    }

    let sectors_by_id = provisional_map
        .sectors
        .iter()
        .map(|sector| (sector.sector_id.as_str(), sector))
        .collect::<HashMap<_, _>>();

    for sector_id in &journal.completed_sectors {
        let Some(sector) = sectors_by_id.get(sector_id.as_str()) else {
            return Err(format!("completed sector {sector_id} is not present"));
        };
        ensure_cached_sector_artifacts_available(blobs_dir, sector, sector.proofs.len())?;
    }

    let Some(active) = journal.active_sector.as_ref() else {
        return Ok(());
    };
    if dirty_sector_ids.get(journal.completed_sectors.len()) != Some(&active.sector_id) {
        return Err("active sector does not follow the completed prefix".to_string());
    }

    let Some(sector) = sectors_by_id.get(active.sector_id.as_str()) else {
        return Err(format!("active sector {} is not present", active.sector_id));
    };
    if active.sector_member_count != sector.proofs.len() {
        return Err("active sector member count changed".to_string());
    }
    if active.next_member_offset > sector.proofs.len() {
        return Err("active sector offset is out of range".to_string());
    }
    let expected_next_path = sector
        .proofs
        .get(active.next_member_offset)
        .map(|proof| proof.relative_path.clone());
    if active.next_member_relative_path != expected_next_path {
        return Err("active sector next member path mismatch".to_string());
    }
    ensure_cached_sector_artifacts_available(blobs_dir, sector, active.next_member_offset)
}

fn ensure_cached_sector_artifacts_available(
    blobs_dir: &Path,
    sector: &crate::cache::SectorRecord,
    count: usize,
) -> std::result::Result<(), String> {
    if count == 0 {
        return Ok(());
    }

    load_sector_artifacts_from_cache_prefix(blobs_dir, sector, count)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn persist_manifest_checkpoint(manifest: &Manifest, cache_paths: &CachePaths) -> Result<()> {
    manifest.save(&cache_paths.manifest_path)
}

fn load_file_artifact_with_tracking(
    root: &Path,
    path: &Path,
    manifest: &mut Manifest,
    cache_paths: Option<&CachePaths>,
    telemetry: &crate::system::Telemetry,
    artifacts: &mut Vec<ContextArtifact>,
) {
    match load_file_artifact(root, path, manifest, cache_paths, telemetry) {
        Ok(Some(artifact)) => record_loaded_artifact(artifacts, artifact, telemetry),
        Ok(None) => {
            telemetry.skipped_artifacts.fetch_add(1, Ordering::Relaxed);
        }
        Err(error) => {
            telemetry.skipped_artifacts.fetch_add(1, Ordering::Relaxed);
            tracing::warn!(
                path = %path.display(),
                error = %error,
                "skipping unreadable artifact during corpus load"
            );
        }
    }
}

fn record_loaded_artifact(
    artifacts: &mut Vec<ContextArtifact>,
    artifact: ContextArtifact,
    telemetry: &crate::system::Telemetry,
) {
    telemetry
        .total_segments
        .fetch_add(artifact.segments.len(), Ordering::Relaxed);
    artifacts.push(artifact);
}

fn persist_sector_map_and_shards(
    root: &Path,
    cache_base: &Path,
    file_artifacts: &[ContextArtifact],
    cached_sector_map: &SectorMap,
    telemetry: &crate::system::Telemetry,
) -> Result<()> {
    let inputs = file_artifacts
        .iter()
        .filter(|artifact| {
            matches!(
                artifact.kind,
                ContextArtifactKind::File | ContextArtifactKind::ProjectDocument
            )
        })
        .map(artifact_to_sector_member_input)
        .collect::<Vec<_>>();
    let mut final_sector_map =
        SectorMap::build_for_root(root, inputs, SectorPartitionStrategy::default())?;
    let artifacts_by_relative_path = file_artifacts
        .iter()
        .filter(|artifact| {
            matches!(
                artifact.kind,
                ContextArtifactKind::File | ContextArtifactKind::ProjectDocument
            )
        })
        .map(|artifact| (sector_relative_path(root, &artifact.path), artifact))
        .collect::<HashMap<_, _>>();
    let cached_sectors_by_id = cached_sector_map
        .sectors
        .iter()
        .map(|sector| (sector.sector_id.clone(), sector))
        .collect::<HashMap<_, _>>();

    for sector in &mut final_sector_map.sectors {
        if let Some(cached_sector) = cached_sectors_by_id.get(&sector.sector_id)
            && cached_sector.membership.proof_fingerprint == sector.membership.proof_fingerprint
            && sector_bm25_shard_available(cache_base, root, cached_sector)
        {
            telemetry.sector_cache_hits.fetch_add(1, Ordering::Relaxed);
            sector.shards = cached_sector.shards.clone();
            continue;
        }

        let sector_artifacts = sector
            .proofs
            .iter()
            .filter_map(|proof| {
                artifacts_by_relative_path
                    .get(&proof.relative_path)
                    .copied()
            })
            .cloned()
            .collect::<Vec<_>>();
        let shard_key = sector.membership.proof_fingerprint.clone();
        let shard = Bm25Index::build(&sector_artifacts);
        save_sector_bm25_shard(cache_base, root, &sector.sector_id, &shard_key, &shard)?;
        telemetry.sector_rebuilds.fetch_add(1, Ordering::Relaxed);
        telemetry
            .sector_shard_builds
            .fetch_add(1, Ordering::Relaxed);
        sector.shards.bm25 = Some(SectorLexicalShardRef {
            format: SectorLexicalShardFormat::Bm25Bincode,
            key: shard_key,
        });
    }

    final_sector_map.save_for_root(cache_base, root)
}

fn artifact_to_sector_member_input(artifact: &ContextArtifact) -> SectorMemberInput {
    SectorMemberInput::filesystem(
        artifact.path.clone(),
        artifact_blob_key(artifact).unwrap_or_else(|| synthetic_cache_key(artifact)),
    )
}

fn artifact_blob_key(artifact: &ContextArtifact) -> Option<String> {
    artifact
        .id
        .rsplit_once(':')
        .map(|(_, hash)| hash.to_string())
}

fn dirty_sector_placeholder(current: Option<&CacheEntry>) -> String {
    current
        .map(|current| {
            format!(
                "dirty:{}:{}:{}:{}",
                current.inode, current.mtime_secs, current.mtime_nanos, current.size
            )
        })
        .unwrap_or_else(|| "dirty:unknown".to_string())
}

fn load_sector_artifacts_from_cache(
    blobs_dir: &Path,
    sector: &crate::cache::SectorRecord,
) -> Result<Vec<ContextArtifact>> {
    load_sector_artifacts_from_cache_prefix(blobs_dir, sector, sector.proofs.len())
}

fn load_sector_artifacts_from_cache_prefix(
    blobs_dir: &Path,
    sector: &crate::cache::SectorRecord,
    count: usize,
) -> Result<Vec<ContextArtifact>> {
    sector
        .proofs
        .iter()
        .take(count)
        .map(|proof| load_blob(blobs_dir, &proof.artifact_blob_key))
        .collect()
}

fn sector_bm25_shard_available(
    cache_base: &Path,
    root: &Path,
    sector: &crate::cache::SectorRecord,
) -> bool {
    let Some(shard) = sector.shards.bm25.as_ref() else {
        return false;
    };
    sector_bm25_shard_cache_path(cache_base, root, &sector.sector_id, &shard.key).exists()
}

pub fn load_sector_bm25_shard(
    cache_base: &Path,
    root: &Path,
    sector_id: &str,
    shard_key: &str,
) -> Result<Option<Bm25Index>> {
    load_locked_bincode(&sector_bm25_shard_cache_path(
        cache_base, root, sector_id, shard_key,
    ))
}

pub fn save_sector_bm25_shard(
    cache_base: &Path,
    root: &Path,
    sector_id: &str,
    shard_key: &str,
    index: &Bm25Index,
) -> Result<()> {
    save_locked_bincode(
        &sector_bm25_shard_cache_path(cache_base, root, sector_id, shard_key),
        index,
    )
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use tempfile::tempdir;

    use super::*;
    use crate::cache::{BREADCRUMB_STALE_AFTER_SECS, breadcrumb_cache_path};
    use crate::search::{SearchCoverageMode, SearchCoverageSnapshot};

    fn write_breadcrumb_corpus(root: &Path) {
        std::fs::write(root.join("alpha.txt"), "alpha retrieval cache sector").expect("alpha");
        std::fs::write(root.join("beta.txt"), "beta retrieval cache sector").expect("beta");
        std::fs::write(root.join("gamma.txt"), "gamma retrieval cache sector").expect("gamma");
    }

    fn write_resume_corpus(root: &Path) {
        for index in 0..64 {
            let name = format!("doc-{index:02}.txt");
            let contents = format!("retrieval breadcrumb sector document {index}");
            std::fs::write(root.join(name), contents).expect("write resume corpus file");
        }
    }

    #[test]
    fn persists_breadcrumb_journal_during_dirty_sector_rebuild() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_breadcrumb_corpus(corpus.path());

        let telemetry = crate::system::Telemetry::new();
        load_search_corpus(
            corpus.path(),
            None,
            0,
            None,
            &telemetry,
            &[],
            Some(cache.path()),
        )
        .expect("initial load");

        std::fs::write(
            corpus.path().join("alpha.txt"),
            "alpha retrieval cache sector changed",
        )
        .expect("rewrite dirty file");

        let file_paths = collect_file_paths(corpus.path(), None);
        let cache_paths = CachePaths::for_root(cache.path(), corpus.path());
        let mut manifest = Manifest::load(&cache_paths.manifest_path).expect("manifest");
        let mut processed = 0usize;
        let mut checkpoint_calls = 0usize;
        let mut hooks = RebuildHooks {
            after_breadcrumb_checkpoint: Some(&mut |journal| {
                checkpoint_calls += 1;
                if journal
                    .active_sector
                    .as_ref()
                    .is_some_and(|active| active.next_member_offset >= 1)
                {
                    return Err(anyhow::anyhow!("interrupt for breadcrumb test"));
                }
                Ok(())
            }),
        };

        let error = load_file_artifacts_with_sector_cache_impl(
            corpus.path(),
            &file_paths,
            &mut manifest,
            cache.path(),
            &cache_paths,
            &telemetry,
            &mut || {
                processed += 1;
            },
            &mut hooks,
        )
        .expect_err("should interrupt after breadcrumb checkpoint");
        assert!(error.to_string().contains("interrupt for breadcrumb test"));
        assert!(checkpoint_calls >= 2);
        assert!(processed >= 1);
        assert!(processed < file_paths.len());

        let journal = BreadcrumbJournal::load_for_root(cache.path(), corpus.path())
            .expect("load breadcrumb journal")
            .expect("breadcrumb journal should exist");
        assert!(!journal.run_id.is_empty());
        assert!(!journal.dirty_sectors.is_empty());
        assert!(journal.updated_at_unix_secs > 0);
        assert!(journal.completed_sectors.is_empty());
        let active = journal
            .active_sector
            .expect("active sector should be recorded");
        assert_eq!(active.next_member_offset, 1);
        assert!(active.sector_member_count >= active.next_member_offset);
    }

    #[test]
    fn frontier_snapshot_reflects_warm_sector_reuse() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_resume_corpus(corpus.path());

        let initial = crate::system::Telemetry::new();
        load_search_corpus(
            corpus.path(),
            None,
            0,
            None,
            &initial,
            &[],
            Some(cache.path()),
        )
        .expect("initial load");

        let warm = crate::system::Telemetry::new();
        load_search_corpus(corpus.path(), None, 0, None, &warm, &[], Some(cache.path()))
            .expect("warm load");

        let sector_map = SectorMap::load_for_root(cache.path(), corpus.path()).expect("sector map");
        let frontier = warm.frontier_snapshot();
        assert_eq!(frontier.total_sector_count, sector_map.sectors.len());
        assert_eq!(frontier.mounted_sector_count, sector_map.sectors.len());
        assert_eq!(frontier.reused_sector_count, sector_map.sectors.len());
        assert_eq!(frontier.dirty_sector_count, 0);
        assert_eq!(frontier.completed_dirty_sector_count, 0);
        assert_eq!(frontier.rebuilding_sector_count, 0);
        assert!(frontier.active_rebuild.is_none());
    }

    #[test]
    fn resumes_interrupted_dirty_sector_rebuilds_from_breadcrumb_state() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_resume_corpus(corpus.path());

        let initial_telemetry = crate::system::Telemetry::new();
        load_search_corpus(
            corpus.path(),
            None,
            0,
            None,
            &initial_telemetry,
            &[],
            Some(cache.path()),
        )
        .expect("initial load");

        let before = SectorMap::load_for_root(cache.path(), corpus.path()).expect("sector map");
        assert!(before.sectors.len() > 1);
        let dirty_sector = before
            .sectors
            .iter()
            .find(|sector| sector.proofs.len() > 1)
            .expect("sector with multiple members");
        let changed_path = resolve_sector_member_path(
            corpus.path(),
            &dirty_sector
                .proofs
                .first()
                .expect("dirty proof")
                .relative_path,
        );
        std::fs::write(
            &changed_path,
            "retrieval breadcrumb sector document changed to force rebuild",
        )
        .expect("rewrite dirty file");

        let file_paths = collect_file_paths(corpus.path(), None);
        let cache_paths = CachePaths::for_root(cache.path(), corpus.path());
        let mut manifest = Manifest::load(&cache_paths.manifest_path).expect("manifest");
        let interrupted_telemetry = crate::system::Telemetry::new();
        let mut hooks = RebuildHooks {
            after_breadcrumb_checkpoint: Some(&mut |journal| {
                if journal
                    .active_sector
                    .as_ref()
                    .is_some_and(|active| active.next_member_offset >= 1)
                {
                    return Err(anyhow::anyhow!("interrupt for breadcrumb resume test"));
                }
                Ok(())
            }),
        };

        let error = load_file_artifacts_with_sector_cache_impl(
            corpus.path(),
            &file_paths,
            &mut manifest,
            cache.path(),
            &cache_paths,
            &interrupted_telemetry,
            &mut || {},
            &mut hooks,
        )
        .expect_err("should interrupt dirty rebuild");
        assert!(
            error
                .to_string()
                .contains("interrupt for breadcrumb resume test")
        );
        let interrupted_frontier = interrupted_telemetry.frontier_snapshot();
        let interrupted_coverage = SearchCoverageSnapshot::from_frontier(&interrupted_frontier);
        assert_eq!(interrupted_frontier.rebuilding_sector_count, 1);
        assert!(interrupted_frontier.active_rebuild.is_some());
        assert!(interrupted_frontier.dirty_sector_count > 0);
        assert_eq!(interrupted_coverage.mode, SearchCoverageMode::Converging);

        let resumed_telemetry = crate::system::Telemetry::new();
        let loaded = load_search_corpus(
            corpus.path(),
            None,
            0,
            None,
            &resumed_telemetry,
            &[],
            Some(cache.path()),
        )
        .expect("resume from breadcrumb");

        assert_eq!(loaded.indexed_artifacts, file_paths.len());
        assert_eq!(
            resumed_telemetry
                .breadcrumb_resume_hits
                .load(Ordering::Relaxed),
            1
        );
        assert_eq!(
            resumed_telemetry
                .breadcrumb_recovery_discards
                .load(Ordering::Relaxed),
            0
        );
        assert!(resumed_telemetry.sector_cache_hits.load(Ordering::Relaxed) > 0);
        let resumed_frontier = resumed_telemetry.frontier_snapshot();
        let resumed_coverage = SearchCoverageSnapshot::from_frontier(&resumed_frontier);
        assert_eq!(resumed_frontier.dirty_sector_count, 0);
        assert!(resumed_frontier.completed_dirty_sector_count > 0);
        assert!(resumed_frontier.resumed_sector_count > 0);
        assert_eq!(resumed_frontier.rebuilding_sector_count, 0);
        assert!(resumed_frontier.active_rebuild.is_none());
        assert_eq!(resumed_coverage.mode, SearchCoverageMode::Sealed);
        assert!(
            BreadcrumbJournal::load_for_root(cache.path(), corpus.path())
                .expect("load breadcrumb after resume")
                .is_none()
        );
    }

    #[test]
    fn discards_stale_breadcrumb_journal_before_restart() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_resume_corpus(corpus.path());

        let initial_telemetry = crate::system::Telemetry::new();
        load_search_corpus(
            corpus.path(),
            None,
            0,
            None,
            &initial_telemetry,
            &[],
            Some(cache.path()),
        )
        .expect("initial load");

        let sector_map = SectorMap::load_for_root(cache.path(), corpus.path()).expect("sector map");
        let dirty_sector = sector_map
            .sectors
            .iter()
            .find(|sector| !sector.proofs.is_empty())
            .expect("dirty sector");
        let changed_path = resolve_sector_member_path(
            corpus.path(),
            &dirty_sector
                .proofs
                .first()
                .expect("dirty proof")
                .relative_path,
        );
        std::fs::write(&changed_path, "retrieval breadcrumb stale journal change")
            .expect("rewrite dirty file");

        let mut journal = BreadcrumbJournal::new(
            corpus_cache_key(corpus.path()),
            "stale-run".to_string(),
            vec![dirty_sector.sector_id.clone()],
        );
        journal.updated_at_unix_secs = current_unix_secs() - BREADCRUMB_STALE_AFTER_SECS - 1;
        journal
            .save_for_root(cache.path(), corpus.path())
            .expect("save stale breadcrumb");

        let telemetry = crate::system::Telemetry::new();
        load_search_corpus(
            corpus.path(),
            None,
            0,
            None,
            &telemetry,
            &[],
            Some(cache.path()),
        )
        .expect("load after stale breadcrumb");

        assert_eq!(telemetry.breadcrumb_resume_hits.load(Ordering::Relaxed), 0);
        assert_eq!(
            telemetry
                .breadcrumb_recovery_discards
                .load(Ordering::Relaxed),
            1
        );
        assert!(
            BreadcrumbJournal::load_for_root(cache.path(), corpus.path())
                .expect("load breadcrumb after stale discard")
                .is_none()
        );
    }

    #[test]
    fn discards_corrupt_breadcrumb_journal_before_restart() {
        let corpus = tempdir().expect("temp corpus");
        let cache = tempdir().expect("temp cache");
        write_resume_corpus(corpus.path());

        let initial_telemetry = crate::system::Telemetry::new();
        load_search_corpus(
            corpus.path(),
            None,
            0,
            None,
            &initial_telemetry,
            &[],
            Some(cache.path()),
        )
        .expect("initial load");

        let sector_map = SectorMap::load_for_root(cache.path(), corpus.path()).expect("sector map");
        let dirty_sector = sector_map
            .sectors
            .iter()
            .find(|sector| !sector.proofs.is_empty())
            .expect("dirty sector");
        let changed_path = resolve_sector_member_path(
            corpus.path(),
            &dirty_sector
                .proofs
                .first()
                .expect("dirty proof")
                .relative_path,
        );
        std::fs::write(&changed_path, "retrieval breadcrumb corrupt journal change")
            .expect("rewrite dirty file");

        let path = breadcrumb_cache_path(cache.path(), corpus.path());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create breadcrumb parent");
        }
        std::fs::write(&path, b"not-bincode").expect("write corrupt breadcrumb");

        let telemetry = crate::system::Telemetry::new();
        load_search_corpus(
            corpus.path(),
            None,
            0,
            None,
            &telemetry,
            &[],
            Some(cache.path()),
        )
        .expect("load after corrupt breadcrumb");

        assert_eq!(telemetry.breadcrumb_resume_hits.load(Ordering::Relaxed), 0);
        assert_eq!(
            telemetry
                .breadcrumb_recovery_discards
                .load(Ordering::Relaxed),
            1
        );
        assert!(
            BreadcrumbJournal::load_for_root(cache.path(), corpus.path())
                .expect("load breadcrumb after corrupt discard")
                .is_none()
        );
    }
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
    telemetry
        .fresh_artifact_builds
        .fetch_add(1, Ordering::Relaxed);

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

    telemetry
        .fresh_artifact_builds
        .fetch_add(1, Ordering::Relaxed);

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
        Self {
            manifest_path: base
                .join("artifacts")
                .join("manifests")
                .join(corpus_cache_key(root)),
            blobs_dir: base.join("artifacts").join("blobs"),
        }
    }
}
