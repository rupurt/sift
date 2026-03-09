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

    use rayon::prelude::*;

    let results: Vec<Option<Document>> = files_to_process
        .into_par_iter()
        .map(|path| {
            let extracted = match extract_path(&path) {
                Ok(Some(extracted)) => extracted,
                Ok(None) | Err(_) => {
                    return None;
                }
            };

            let id = match path.file_stem().and_then(|stem| stem.to_str()) {
                Some(id) => id.to_string(),
                None => {
                    return None;
                }
            };

            Some(index_document(id, path, extracted))
        })
        .collect();

    let mut documents = Vec::new();
    let mut total_bytes = 0_u64;
    let mut skipped_files = 0_usize;

    for result in results {
        if let Some(doc) = result {
            total_bytes += doc.text.len() as u64;
            documents.push(doc);
        } else {
            skipped_files += 1;
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

pub fn load_search_corpus(
    root: &Path,
    ignore: Option<&Ignore>,
    verbose: u8,
    dense: Option<&DenseReranker>,
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
        crate::trace!(2, verbose, "    directory walk found {} files in {:.2?}", files_to_process.len(), walk_start.elapsed());
    } else {
        bail!(
            "search path '{}' is neither a regular file nor directory",
            root.display()
        );
    }

    use rayon::prelude::*;

    enum ProcessResult {
        Hit(Document),
        HashHit(Document, PathBuf, crate::cache::CacheEntry, String),
        Miss(Document, PathBuf, crate::cache::CacheEntry, String),
        Skip,
    }

    let process_start = std::time::Instant::now();
    let results: Vec<ProcessResult> = files_to_process
        .into_par_iter()
        .map(|path| {
            let Ok(heuristics) = get_file_heuristics(&path) else {
                return ProcessResult::Skip;
            };

            // Fast path: heuristics match manifest
            if let Some(hash) = manifest.check_heuristics(&path, &heuristics) {
                if let Ok(mut doc) = load_blob(&blobs_dir, &hash) {
                    crate::trace!(3, verbose, "      cache hit (heuristic): {}", path.display());
                    doc.id = path.display().to_string();
                    doc.path = path.to_path_buf();
                    return ProcessResult::Hit(doc);
                }
            }

            // Medium path: compute blake3 and check global blob store
            if let Ok(hash) = hash_file(&path) {
                if let Ok(mut doc) = load_blob(&blobs_dir, &hash) {
                    crate::trace!(3, verbose, "      cache hit (hash): {}", path.display());
                    doc.id = path.display().to_string();
                    doc.path = path.to_path_buf();
                    return ProcessResult::HashHit(doc, path, heuristics, hash);
                }

                // Slow path: True miss, must extract and embed
                crate::trace!(3, verbose, "      cache miss: {}", path.display());
                if let Ok(Some(extracted)) = extract_path(&path) {
                    let id = path.display().to_string();
                    let mut doc = index_document(id, path.clone(), extracted);
                    
                    if let Some(dense) = dense {
                        let texts: Vec<_> = doc.segments.iter().map(|s| s.text.clone()).collect();
                        if let Ok(embeddings) = dense.embed_batch(&texts) {
                            for (i, embedding) in embeddings.into_iter().enumerate() {
                                doc.segments[i].embedding = Some(embedding);
                            }
                        }
                    }

                    if save_blob(&blobs_dir, &hash, &doc).is_ok() {
                        return ProcessResult::Miss(doc, path, heuristics, hash);
                    } else {
                        return ProcessResult::Hit(doc); // Still return it even if save failed
                    }
                } else {
                    return ProcessResult::Skip;
                }
            } else {
                return ProcessResult::Skip;
            }
        })
        .collect();

    let mut documents = Vec::new();
    let mut total_bytes = 0_u64;
    let mut skipped_files = 0_usize;
    let mut manifest_updated = false;

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
                manifest.update(path, heuristics, hash);
                manifest_updated = true;
                total_bytes += doc.text.len() as u64;
                documents.push(doc);
            }
            ProcessResult::Skip => {
                skipped_files += 1;
            }
        }
    }

    crate::trace!(2, verbose, "    parallel processing took: {:.2?}", process_start.elapsed());

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
