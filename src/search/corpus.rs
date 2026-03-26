use std::collections::HashMap;
use std::path::Path;

use anyhow::{Result, bail};
use rayon::prelude::*;
use walkdir::WalkDir;

use super::domain::{Document, Embedder, Ignore, LoadedCorpus};

pub fn load_search_corpus(
    root: &Path,
    ignore: Option<&Ignore>,
    _verbose: u8,
    _dense: Option<&dyn Embedder>,
    _telemetry: &crate::system::Telemetry,
    _cache_base: Option<&Path>,
) -> Result<LoadedCorpus> {
    if !root.exists() {
        bail!("search path '{}' does not exist", root.display());
    }

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
        for entry in WalkDir::new(root).sort_by_file_name().into_iter().flatten() {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ignore) = ignore
                    && ignore.is_ignored(path)
                {
                    continue;
                }
                files_to_process.push(path.to_path_buf());
            }
        }
    }

    let total_files = files_to_process.len();
    let results: Vec<Document> = files_to_process
        .into_par_iter()
        .map(|path| {
            let extracted = match crate::extract::extract_path(&path) {
                Ok(Some(document)) => document,
                Ok(None) => return None,
                Err(error) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %error,
                        "skipping unreadable document during search corpus load"
                    );
                    return None;
                }
            };

            let hash = blake3::hash(extracted.text.as_bytes()).to_string();
            let doc_id = format!("{}:{}", path.display(), hash);
            let segments = crate::segment::build_segments(
                &doc_id,
                &path,
                extracted.source_kind,
                &extracted.text,
            );
            let mut terms = HashMap::new();
            for term in crate::search::domain::tokenize(&extracted.text) {
                *terms.entry(term).or_insert(0) += 1;
            }

            Some(Document {
                id: doc_id,
                path: path.clone(),
                source_kind: extracted.source_kind,
                length: extracted.text.len(),
                terms,
                text: extracted.text,
                segments,
            })
        })
        .flatten()
        .collect();

    let mut total_bytes = 0;
    let indexed_files = results.len();
    let skipped_files = total_files.saturating_sub(indexed_files);

    for doc in &results {
        total_bytes += doc.length as u64;
    }

    Ok(LoadedCorpus {
        documents: results,
        total_bytes,
        indexed_files,
        skipped_files,
    })
}
