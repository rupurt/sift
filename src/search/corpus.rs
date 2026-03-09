use anyhow::{Result, bail};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::domain::*;
use crate::config::Ignore;
use crate::extract::extract_path;
use crate::segment::build_segments;

pub fn load_materialized_corpus(
    corpus_dir: &Path,
    ignore: Option<&Ignore>,
) -> Result<LoadedCorpus> {
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

pub fn load_search_corpus(root: &Path, ignore: Option<&Ignore>) -> Result<LoadedCorpus> {
    if !root.exists() {
        bail!("search path '{}' does not exist", root.display());
    }

    let mut documents = Vec::new();
    let mut total_bytes = 0_u64;
    let mut skipped_files = 0_usize;

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
        collect_search_file(root, &mut documents, &mut total_bytes, &mut skipped_files);
    } else if root.is_dir() {
        for entry in WalkDir::new(root).sort_by_file_name() {
            match entry {
                Ok(entry) => {
                    if let Some(ignore) = ignore {
                        if ignore.is_ignored(entry.path()) {
                            continue;
                        }
                    }

                    if entry.file_type().is_file() {
                        collect_search_file(
                            entry.path(),
                            &mut documents,
                            &mut total_bytes,
                            &mut skipped_files,
                        );
                    }
                }
                Err(_) => skipped_files += 1,
            }
        }
    } else {
        bail!(
            "search path '{}' is neither a regular file nor directory",
            root.display()
        );
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

fn collect_search_file(
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

    *total_bytes += extracted.text.len() as u64;
    documents.push(index_document(
        path.display().to_string(),
        path.to_path_buf(),
        extracted,
    ));
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
