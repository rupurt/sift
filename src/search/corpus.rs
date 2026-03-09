use anyhow::{Result, bail};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::domain::*;
use crate::config::Ignore;
use crate::extract::extract_path;
use crate::segment::build_segments;
use crate::cache::{Manifest, get_file_heuristics, hash_file, load_blob, save_blob, cache_dir};

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

    let mut documents = Vec::new();
    let mut total_bytes = 0_u64;
    let mut skipped_files = 0_usize;

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

                collect_materialized_file(
                    entry.path(),
                    &mut documents,
                    &mut total_bytes,
                    &mut skipped_files,
                );
            }
            Err(_) => skipped_files += 1,
        }
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

pub fn load_search_corpus(root: &Path, ignore: Option<&Ignore>, verbose: u8) -> Result<LoadedCorpus> {
    if !root.exists() {
        bail!("search path '{}' does not exist", root.display());
    }

    let mut documents = Vec::new();
    let mut total_bytes = 0_u64;
    let mut skipped_files = 0_usize;

    let manifest_path = get_project_manifest_path(root)?;
    let mut manifest = Manifest::load(&manifest_path).unwrap_or_default();
    let blobs_dir = cache_dir("blobs")?;
    let mut manifest_updated = false;

    if root.is_file() {
        if let Some(ignore) = ignore {
            if ignore.is_ignored(root) {
                return Ok(LoadedCorpus {
                    documents: Vec::new(),
                    total_bytes: 0,
                    indexed_files: 0,
                    skipped_files: 0,
                });
            }
        }
        collect_search_file_cached(
            root,
            &blobs_dir,
            &mut manifest,
            &mut manifest_updated,
            &mut documents,
            &mut total_bytes,
            &mut skipped_files,
            verbose,
        );
    } else if root.is_dir() {
        let walk_start = std::time::Instant::now();
        for entry in WalkDir::new(root).sort_by_file_name() {
            match entry {
                Ok(entry) => {
                    if let Some(ignore) = ignore {
                        if ignore.is_ignored(entry.path()) {
                            continue;
                        }
                    }

                    if entry.file_type().is_file() {
                        collect_search_file_cached(
                            entry.path(),
                            &blobs_dir,
                            &mut manifest,
                            &mut manifest_updated,
                            &mut documents,
                            &mut total_bytes,
                            &mut skipped_files,
                            verbose,
                        );
                    }
                }
                Err(_) => skipped_files += 1,
            }
        }
        crate::trace!(2, verbose, "    directory walk and load took: {:.2?}", walk_start.elapsed());
    } else {
        bail!(
            "search path '{}' is neither a regular file nor directory",
            root.display()
        );
    }

    if manifest_updated {
        let save_start = std::time::Instant::now();
        let _ = manifest.save(&manifest_path);
        crate::trace!(2, verbose, "    manifest updated in {:.2?}", save_start.elapsed());
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

fn collect_search_file_cached(
    path: &Path,
    blobs_dir: &Path,
    manifest: &mut Manifest,
    manifest_updated: &mut bool,
    documents: &mut Vec<Document>,
    total_bytes: &mut u64,
    skipped_files: &mut usize,
    verbose: u8,
) {
    let Ok(heuristics) = get_file_heuristics(path) else {
        *skipped_files += 1;
        return;
    };

    // Fast path: heuristics match manifest
    if let Some(hash) = manifest.check_heuristics(path, &heuristics) {
        if let Ok(mut doc) = load_blob(blobs_dir, &hash) {
            crate::trace!(2, verbose, "    cache hit (heuristic): {}", path.display());
            // Update document ID to match current path, just in case file moved
            doc.id = path.display().to_string();
            doc.path = path.to_path_buf();
            *total_bytes += doc.text.len() as u64;
            documents.push(doc);
            return;
        }
    }

    // Medium path: compute blake3 and check global blob store
    if let Ok(hash) = hash_file(path) {
        if let Ok(mut doc) = load_blob(blobs_dir, &hash) {
            crate::trace!(2, verbose, "    cache hit (hash): {}", path.display());
            manifest.update(path.to_path_buf(), heuristics, hash);
            *manifest_updated = true;
            
            doc.id = path.display().to_string();
            doc.path = path.to_path_buf();
            *total_bytes += doc.text.len() as u64;
            documents.push(doc);
            return;
        }

        // Slow path: True miss, must extract and (later) embed
        crate::trace!(2, verbose, "    cache miss: {}", path.display());
        if let Ok(Some(extracted)) = extract_path(path) {
            let id = path.display().to_string();
            let doc = index_document(id, path.to_path_buf(), extracted);
            
            // Pre-warm the cache. Note: Segments don't have embeddings yet, 
            // but the text extraction work is saved.
            if save_blob(blobs_dir, &hash, &doc).is_ok() {
                manifest.update(path.to_path_buf(), heuristics, hash);
                *manifest_updated = true;
            }

            *total_bytes += doc.text.len() as u64;
            documents.push(doc);
        } else {
            *skipped_files += 1;
        }
    } else {
        *skipped_files += 1;
    }
}

fn collect_materialized_file(
    path: &Path,
    documents: &mut Vec<Document>,
    total_bytes: &mut u64,
    skipped_files: &mut usize,
) {
    let extracted = match extract_path(path) {
        Ok(Some(extracted)) => extracted,
        Ok(None) | Err(_) => {
            *skipped_files += 1;
            return;
        }
    };

    let id = match path.file_stem().and_then(|stem| stem.to_str()) {
        Some(id) => id.to_string(),
        None => {
            *skipped_files += 1;
            return;
        }
    };

    *total_bytes += extracted.text.len() as u64;
    documents.push(index_document(id, path.to_path_buf(), extracted));
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
