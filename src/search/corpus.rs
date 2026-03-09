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
    let total_files = files_to_process.len();

    // Stage 1: Parallel cache check and extraction (lexical only)
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

                // Slow path: True miss, must extract
                if let Ok(Some(extracted)) = extract_path(&path) {
                    let id = path.display().to_string();
                    let doc = index_document(id, path.to_path_buf(), extracted);
                    return ProcessResult::Miss(doc, path, heuristics, hash);
                }
            }
            ProcessResult::Skip
        })
        .collect();

    // Stage 2: Batch vectorize all new documents across the whole corpus
    let mut documents = Vec::new();
    let mut total_bytes = 0_u64;
    let mut skipped_files = 0_usize;
    let mut manifest_updated = false;
    
    // We'll collect documents that need embedding
    let mut docs_to_embed = Vec::new();
    // Indices in the final 'documents' vector for the docs being embedded
    let mut doc_indices = Vec::new();

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
                doc_indices.push(idx);
            }
            ProcessResult::Skip => {
                skipped_files += 1;
            }
        }
    }

    // Perform batch embedding if model is present and we have misses
    if let Some(dense) = dense {
        if !docs_to_embed.is_empty() {
            crate::trace!(1, verbose, "    vectorizing {} new/changed documents...", docs_to_embed.len());
            
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
                if let Ok(embeddings) = dense.embed_batch(&all_texts) {
                    for (i, embedding) in embeddings.into_iter().enumerate() {
                        let (doc_idx, seg_idx) = segment_refs[i];
                        documents[doc_idx].segments[seg_idx].embedding = Some(embedding);
                    }
                    crate::trace!(2, verbose, "    embedded {} segments in {:.2?}", all_texts.len(), embed_start.elapsed());
                }
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

    crate::trace!(2, verbose, "    processing {} files took: {:.2?}", total_files, process_start.elapsed());

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
