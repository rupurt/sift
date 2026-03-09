use anyhow::{Result, bail};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::domain::*;
use crate::config::Ignore;
use crate::extract::extract_path;
use crate::segment::build_segments;
use crate::cache::{Manifest, get_file_heuristics, hash_file, load_blob, save_blob, cache_dir};

use crate::dense::DenseReranker;

enum ProcessResult {
    Hit(Document),
    HashHit(Document, PathBuf, crate::cache::CacheEntry, String),
    Miss(Document, PathBuf, crate::cache::CacheEntry, String),
    Skip,
}

fn get_project_manifest_path(root: &Path) -> Result<PathBuf> {
    let absolute = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let path_str = absolute.to_string_lossy();
    let hash = blake3::hash(path_str.as_bytes()).to_hex().to_string();
    let manifests_dir = cache_dir("manifests")?;
    Ok(manifests_dir.join(format!("{}.bin", hash)))
}

pub fn load_materialized_corpus(
    corpus_dir: &Path,
    ignore: Option<&Ignore>,
) -> Result<LoadedCorpus> {
    // Kept standard for evaluation benchmarking (eval docs change less often and are isolated)
    if !corpus_dir.exists() {
        bail!("corpus path '{}' does not exist", corpus_dir.display());
    }

    let mut files_to_process = Vec::new();

    for entry in WalkDir::new(corpus_dir).sort_by_file_name() {
        match entry {
            Ok(entry) => {
                if !entry.file_type().is_file() {
                    continue;
                }

                if is_benchmark_metadata_file(corpus_dir, entry.path()) {
                    continue;
                }

                if let Some(ignore) = ignore {
                    if ignore.is_ignored(entry.path()) {
                        continue;
                    }
                }

                files_to_process.push(entry.path().to_path_buf());
            }
            Err(_) => {}
        }
    }

    let mut documents = Vec::new();
    let mut total_bytes = 0_u64;
    let mut skipped_files = 0_usize;

    for path in files_to_process {
        let extracted = match extract_path(&path) {
            Ok(Some(extracted)) => extracted,
            Ok(None) | Err(_) => {
                skipped_files += 1;
                continue;
            }
        };

        let id = match path.file_stem().and_then(|stem| stem.to_str()) {
            Some(id) => id.to_string(),
            None => {
                skipped_files += 1;
                continue;
            }
        };

        total_bytes += extracted.text.len() as u64;
        documents.push(index_document(id, path, extracted));
    }

    documents.sort_by(|left, right| left.id.cmp(&right.id));
    let indexed_files = documents.len();

    Ok(LoadedCorpus {
        documents,
        total_bytes,
        indexed_files,
        skipped_files,
    })
}

fn is_benchmark_metadata_file(root: &Path, path: &Path) -> bool {
    if path.extension().and_then(|ext| ext.to_str()) == Some("tsv") {
        return true;
    }

    path.strip_prefix(root)
        .ok()
        .map(|relative| {
            relative
                .components()
                .any(|component| component.as_os_str() == "qrels")
        })
        .unwrap_or(false)
}

pub fn load_search_corpus(
    root: &Path,
    ignore: Option<&Ignore>,
    _verbose: u8,
    dense: Option<&DenseReranker>,
    telemetry: &crate::system::Telemetry,
) -> Result<LoadedCorpus> {
    if !root.exists() {
        bail!("search path '{}' does not exist", root.display());
    }

    let manifest_path = get_project_manifest_path(root)?;
    let mut manifest = Manifest::load(&manifest_path).unwrap_or_default();
    let blobs_dir = cache_dir("blobs")?;
    
    let mut files_to_process = Vec::new();

    if root.is_file() {
        if let Some(ignore) = ignore {
            if !ignore.is_ignored(root) {
                files_to_process.push(root.to_path_buf());
            }
        } else {
            files_to_process.push(root.to_path_buf());
        }
    } else if root.is_dir() {
        let walk_start = std::time::Instant::now();
        for entry in WalkDir::new(root).sort_by_file_name() {
            if let Ok(entry) = entry {
                if let Some(ignore) = ignore {
                    if ignore.is_ignored(entry.path()) {
                        continue;
                    }
                }
                if entry.file_type().is_file() {
                    files_to_process.push(entry.path().to_path_buf());
                }
            }
        }
        tracing::info!("directory walk found {} files in {:.2?}", files_to_process.len(), walk_start.elapsed());
    } else {
        bail!(
            "search path '{}' is neither a regular file nor directory",
            root.display()
        );
    }

    let process_start = std::time::Instant::now();
    let total_files = files_to_process.len();
    telemetry.total_files.fetch_add(total_files, std::sync::atomic::Ordering::Relaxed);

    // Stage 1: Sequential cache check and extraction (lexical only)
    let mut results = Vec::new();
    for path in files_to_process {
        let heuristics = match get_file_heuristics(&path) {
            Ok(h) => h,
            Err(_) => {
                results.push(ProcessResult::Skip);
                continue;
            }
        };

        // Fast path: heuristics match manifest
        if let Some(hash) = manifest.check_heuristics(&path, &heuristics) {
            if let Ok(mut doc) = load_blob(&blobs_dir, &hash) {
                tracing::debug!("cache hit (heuristic): {}", path.display());
                doc.id = path.display().to_string();
                doc.path = path.to_path_buf();
                
                telemetry.heuristic_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                
                // If we need embeddings but they are missing, treat as a partial miss
                if dense.is_some() && doc.segments.iter().any(|s| s.embedding.is_none()) {
                    tracing::info!("cache hit but missing embeddings: {}", path.display());
                    results.push(ProcessResult::Miss(doc, path, heuristics, hash));
                } else {
                    results.push(ProcessResult::Hit(doc));
                }
                continue;
            }
        }

        // Medium path: compute blake3 and check global blob store
        if let Ok(hash) = hash_file(&path) {
            if let Ok(mut doc) = load_blob(&blobs_dir, &hash) {
                tracing::debug!("cache hit (hash): {}", path.display());
                doc.id = path.display().to_string();
                doc.path = path.to_path_buf();

                telemetry.blob_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                // If we need embeddings but they are missing, treat as a partial miss
                if dense.is_some() && doc.segments.iter().any(|s| s.embedding.is_none()) {
                    tracing::info!("cache hit but missing embeddings: {}", path.display());
                    results.push(ProcessResult::Miss(doc, path, heuristics, hash));
                } else {
                    results.push(ProcessResult::HashHit(doc, path, heuristics, hash));
                }
                continue;
            }

            // Slow path: True miss, must extract
            if let Ok(Some(extracted)) = extract_path(&path) {
                let id = path.display().to_string();
                let doc = index_document(id, path.to_path_buf(), extracted);
                results.push(ProcessResult::Miss(doc, path, heuristics, hash));
                continue;
            }
        }
        results.push(ProcessResult::Skip);
    }

    // Stage 2: Batch vectorize all new documents across the whole corpus
    let mut documents = Vec::new();
    let mut total_bytes = 0_u64;
    let mut skipped_files = 0_usize;
    let mut manifest_updated = false;
    
    // We'll collect documents that need embedding
    let mut docs_to_embed = Vec::new();

    for result in results {
        match result {
            ProcessResult::Hit(doc) => {
                total_bytes += doc.text.len() as u64;
                documents.push(doc);
            }
            ProcessResult::HashHit(doc, path, heuristics, hash) => {
                manifest.update(path, heuristics, hash);
                manifest_updated = true;
                total_bytes += doc.text.len() as u64;
                documents.push(doc);
            }
            ProcessResult::Miss(doc, path, heuristics, hash) => {
                let idx = documents.len();
                documents.push(doc);
                docs_to_embed.push((idx, path, heuristics, hash));
            }
            ProcessResult::Skip => {
                skipped_files += 1;
            }
        }
    }

    for doc in &documents {
        telemetry.total_segments.fetch_add(doc.segments.len(), std::sync::atomic::Ordering::Relaxed);
        let embedded = doc.segments.iter().filter(|s| s.embedding.is_some()).count();
        telemetry.embedding_hits.fetch_add(embedded, std::sync::atomic::Ordering::Relaxed);
    }

    // Perform batch embedding if model is present and we have misses
    if let Some(dense) = dense {
        if !docs_to_embed.is_empty() {
            tracing::info!("vectorizing {} new/changed documents...", docs_to_embed.len());
            
            // Flatten all segments from all documents into a single batch
            let mut all_texts = Vec::new();
            let mut segment_refs = Vec::new(); // (doc_idx, segment_idx)

            for (doc_idx, _, _, _) in &docs_to_embed {
                let doc = &documents[*doc_idx];
                for (seg_idx, segment) in doc.segments.iter().enumerate() {
                    all_texts.push(segment.text.clone());
                    segment_refs.push((*doc_idx, seg_idx));
                }
            }

            if !all_texts.is_empty() {
                let embed_start = std::time::Instant::now();
                const LOAD_BATCH_SIZE: usize = 128;
                let mut current_idx = 0;
                
                for chunk in all_texts.chunks(LOAD_BATCH_SIZE) {
                    if let Ok(embeddings) = dense.embed_batch(chunk) {
                        for embedding in embeddings {
                            let (doc_idx, seg_idx) = segment_refs[current_idx];
                            documents[doc_idx].segments[seg_idx].embedding = Some(embedding);
                            current_idx += 1;
                        }
                    } else {
                        current_idx += chunk.len();
                    }
                }
                tracing::info!("embedded {} segments in {:.2?}", all_texts.len(), embed_start.elapsed());
            }

            // Save new blobs and update manifest after embedding
            for (doc_idx, path, heuristics, hash) in docs_to_embed {
                let doc = &documents[doc_idx];
                if save_blob(&blobs_dir, &hash, doc).is_ok() {
                    manifest.update(path, heuristics, hash);
                    manifest_updated = true;
                }
                total_bytes += doc.text.len() as u64;
            }
        }
    } else {
        // If no model, still count the bytes for the misses
        for (doc_idx, path, heuristics, hash) in docs_to_embed {
            let doc = &documents[doc_idx];
            if save_blob(&blobs_dir, &hash, doc).is_ok() {
                manifest.update(path, heuristics, hash);
                manifest_updated = true;
            }
            total_bytes += doc.text.len() as u64;
        }
    }

    tracing::info!("processing {} files took: {:.2?}", total_files, process_start.elapsed());

    if manifest_updated {
        let save_start = std::time::Instant::now();
        let _ = manifest.save(&manifest_path);
        tracing::debug!("manifest updated in {:.2?}", save_start.elapsed());
    }

    documents.sort_by(|left, right| left.id.cmp(&right.id));
    let indexed_files = documents.len();

    Ok(LoadedCorpus {
        documents,
        total_bytes,
        indexed_files,
        skipped_files,
    })
}







fn index_document(
    id: String,
    path: PathBuf,
    extracted: crate::extract::ExtractedDocument,
) -> Document {
    let segments = build_segments(&id, &path, extracted.source_kind, &extracted.text);
    let terms = tokenize(&extracted.text)
        .into_iter()
        .fold(HashMap::new(), |mut acc, term| {
            *acc.entry(term).or_insert(0) += 1;
            acc
        });
    let length = terms.values().sum();

    Document {
        id,
        path,
        source_kind: extracted.source_kind,
        length,
        terms,
        text: extracted.text,
        segments,
    }
}
