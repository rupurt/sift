use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::dense::{DenseModelSpec, DenseReranker};
use crate::extract::{SourceKind, extract_path};
use crate::segment::{Segment, build_segments};
use crate::vector::retrieve_semantic_documents;

pub const DEFAULT_RESULT_LIMIT: usize = 10;
pub const DEFAULT_HYBRID_SHORTLIST: usize = 8;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    Bm25,
    Hybrid,
}

impl Engine {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bm25 => "bm25",
            Self::Hybrid => "hybrid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub engine: Engine,
    pub query: String,
    pub path: PathBuf,
    pub limit: usize,
    pub shortlist: usize,
    pub dense_model: DenseModelSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResponse {
    pub engine: Engine,
    pub root: String,
    pub indexed_files: usize,
    pub skipped_files: usize,
    pub results: Vec<SearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchHit {
    pub path: String,
    pub rank: usize,
    pub score: f64,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub struct LoadedCorpus {
    pub documents: Vec<Document>,
    pub total_bytes: u64,
    pub indexed_files: usize,
    pub skipped_files: usize,
}

impl LoadedCorpus {
    fn document_by_id(&self, id: &str) -> Option<&Document> {
        self.documents.iter().find(|document| document.id == id)
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub path: PathBuf,
    pub source_kind: SourceKind,
    pub length: usize,
    pub terms: HashMap<String, usize>,
    text: String,
    segments: Vec<Segment>,
}

impl Document {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoredDocument {
    pub id: String,
    pub path: PathBuf,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankedDocument {
    pub id: String,
    pub path: PathBuf,
    pub score: f64,
    pub bm25_score: f64,
    pub dense_score: Option<f64>,
}

pub struct Bm25Index {
    documents: Vec<Document>,
    doc_freq: HashMap<String, usize>,
    avg_doc_len: f64,
}

impl Bm25Index {
    pub fn build(documents: &[Document]) -> Self {
        let mut doc_freq = HashMap::new();
        let total_len = documents
            .iter()
            .map(|document| document.length)
            .sum::<usize>();

        for document in documents {
            let unique_terms = document.terms.keys().collect::<HashSet<_>>();
            for term in unique_terms {
                *doc_freq.entry(term.clone()).or_insert(0) += 1;
            }
        }

        let avg_doc_len = if documents.is_empty() {
            0.0
        } else {
            total_len as f64 / documents.len() as f64
        };

        Self {
            documents: documents.to_vec(),
            doc_freq,
            avg_doc_len,
        }
    }

    pub fn score(&self, query: &str) -> Vec<ScoredDocument> {
        if self.documents.is_empty() {
            return Vec::new();
        }

        let mut query_terms = tokenize(query);
        query_terms.sort();
        query_terms.dedup();

        let total_docs = self.documents.len() as f64;
        let mut ranked = Vec::with_capacity(self.documents.len());

        for document in &self.documents {
            let mut score = 0.0;

            for term in &query_terms {
                let tf = document.terms.get(term).copied().unwrap_or(0) as f64;
                if tf == 0.0 {
                    continue;
                }

                let doc_freq = self.doc_freq.get(term).copied().unwrap_or(0) as f64;
                let idf = ((total_docs - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln();
                let length_ratio = if self.avg_doc_len > 0.0 {
                    document.length as f64 / self.avg_doc_len
                } else {
                    1.0
                };
                let denominator = tf + 1.5 * (1.0 - 0.75 + 0.75 * length_ratio);
                score += idf * (tf * (1.5 + 1.0) / denominator);
            }

            ranked.push(ScoredDocument {
                id: document.id.clone(),
                path: document.path.clone(),
                score,
            });
        }

        ranked.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.id.cmp(&right.id))
        });
        ranked
    }
}

pub fn run_search(request: &SearchRequest) -> Result<SearchResponse> {
    let corpus = load_search_corpus(&request.path)?;
    let index = Bm25Index::build(&corpus.documents);
    let dense = match request.engine {
        Engine::Bm25 => None,
        Engine::Hybrid => Some(DenseReranker::load(request.dense_model.clone())?),
    };
    let results = rank_corpus(
        &corpus,
        &index,
        dense.as_ref(),
        &request.query,
        request.engine,
        request.limit,
        request.shortlist,
    )?
    .into_iter()
    .enumerate()
    .map(|(index, result)| SearchHit {
        path: result.path.display().to_string(),
        rank: index + 1,
        score: result.score,
        snippet: build_snippet(
            corpus
                .document_by_id(&result.id)
                .map(Document::text)
                .unwrap_or_default(),
        ),
    })
    .collect::<Vec<_>>();

    Ok(SearchResponse {
        engine: request.engine,
        root: request.path.display().to_string(),
        indexed_files: corpus.indexed_files,
        skipped_files: corpus.skipped_files,
        results,
    })
}

pub fn rank_corpus(
    corpus: &LoadedCorpus,
    index: &Bm25Index,
    dense: Option<&DenseReranker>,
    query: &str,
    engine: Engine,
    limit: usize,
    _shortlist: usize,
) -> Result<Vec<RankedDocument>> {
    let lexical = index
        .score(query)
        .into_iter()
        .filter(|document| document.score > 0.0)
        .collect::<Vec<_>>();
    let limit = limit.max(1);

    match engine {
        Engine::Bm25 => Ok(lexical
            .into_iter()
            .take(limit)
            .map(|document| RankedDocument {
                id: document.id,
                path: document.path,
                score: document.score,
                bm25_score: document.score,
                dense_score: None,
            })
            .collect()),
        Engine::Hybrid => {
            let dense =
                dense.ok_or_else(|| anyhow::anyhow!("hybrid search requires a dense scorer"))?;
            let segments = corpus
                .documents
                .iter()
                .flat_map(|document| document.segments().iter().cloned())
                .collect::<Vec<_>>();
            if segments.is_empty() {
                return Ok(Vec::new());
            }

            Ok(retrieve_semantic_documents(dense, query, &segments, limit)?
                .into_iter()
                .map(|document| RankedDocument {
                    id: document.id,
                    path: document.path,
                    score: document.score,
                    bm25_score: 0.0,
                    dense_score: Some(document.score),
                })
                .collect())
        }
    }
}

pub fn render_search_response(response: &SearchResponse, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(response)?),
        OutputFormat::Text => render_text_response(response),
    }
}

pub fn load_materialized_corpus(corpus_dir: &Path) -> Result<LoadedCorpus> {
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

pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if ch.is_alphanumeric() {
            current.extend(ch.to_lowercase());
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn load_search_corpus(root: &Path) -> Result<LoadedCorpus> {
    if !root.exists() {
        bail!("search path '{}' does not exist", root.display());
    }

    let mut documents = Vec::new();
    let mut total_bytes = 0_u64;
    let mut skipped_files = 0_usize;

    if root.is_file() {
        collect_search_file(root, &mut documents, &mut total_bytes, &mut skipped_files);
    } else if root.is_dir() {
        for entry in WalkDir::new(root).sort_by_file_name() {
            match entry {
                Ok(entry) => {
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

fn render_text_response(response: &SearchResponse) -> Result<String> {
    let mut output = String::new();
    writeln!(&mut output, "engine: {}", response.engine.label())?;
    writeln!(&mut output, "root: {}", response.root)?;
    writeln!(&mut output, "indexed_files: {}", response.indexed_files)?;
    writeln!(&mut output, "skipped_files: {}", response.skipped_files)?;

    if response.results.is_empty() {
        writeln!(&mut output, "results: 0")?;
        writeln!(&mut output)?;
        writeln!(&mut output, "no matching results")?;
        return Ok(output.trim_end().to_string());
    }

    writeln!(&mut output, "results: {}", response.results.len())?;
    writeln!(&mut output)?;

    for hit in &response.results {
        writeln!(&mut output, "{}. {}", hit.rank, hit.path)?;
        writeln!(&mut output, "   score: {:.4}", hit.score)?;
        if !hit.snippet.is_empty() {
            writeln!(&mut output, "   snippet: {}", hit.snippet)?;
        }
    }

    Ok(output.trim_end().to_string())
}

fn build_snippet(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return String::new();
    }

    const LIMIT: usize = 160;
    let mut snippet = collapsed.chars().take(LIMIT).collect::<String>();
    if collapsed.chars().count() > LIMIT {
        snippet.push_str("...");
    }
    snippet
}

#[cfg(test)]
mod tests {
    use std::fs as stdfs;
    use std::path::Path;

    use tempfile::tempdir;

    use crate::dense::DenseModelSpec;

    use super::{
        Engine, OutputFormat, SearchRequest, load_search_corpus, render_search_response, run_search,
    };

    mod search {
        use super::*;

        #[test]
        fn bm25_ranks_recursive_utf8_files() {
            let corpus = sample_search_tree();
            let response = run_search(&SearchRequest {
                engine: Engine::Bm25,
                query: "retrieval architecture".to_string(),
                path: corpus.path().to_path_buf(),
                limit: 10,
                shortlist: 10,
                dense_model: DenseModelSpec::default(),
            })
            .expect("search response");

            assert_eq!(response.indexed_files, 3);
            assert_eq!(response.skipped_files, 1);
            assert_eq!(response.results[0].rank, 1);
            assert!(response.results[0].path.ends_with("nested/alpha.txt"));
            assert!(response.results[0].score > response.results[1].score);
        }
    }

    mod cli {
        use super::*;

        #[test]
        fn json_output_contains_result_fields() {
            let corpus = sample_search_tree();
            let response = run_search(&SearchRequest {
                engine: Engine::Bm25,
                query: "retrieval architecture".to_string(),
                path: corpus.path().to_path_buf(),
                limit: 10,
                shortlist: 10,
                dense_model: DenseModelSpec::default(),
            })
            .expect("search response");

            let output =
                render_search_response(&response, OutputFormat::Json).expect("json rendering");
            let parsed = serde_json::from_str::<serde_json::Value>(&output).expect("parse json");
            let first = &parsed["results"][0];

            assert!(first.get("path").is_some());
            assert!(first.get("rank").is_some());
            assert!(first.get("score").is_some());
            assert!(first.get("snippet").is_some());
        }
    }

    mod fs {
        use super::*;

        #[test]
        fn filtering_skips_invalid_utf8_without_crashing() {
            let corpus = sample_search_tree();

            let first = run_search(&SearchRequest {
                engine: Engine::Bm25,
                query: "agent memory".to_string(),
                path: corpus.path().to_path_buf(),
                limit: 10,
                shortlist: 10,
                dense_model: DenseModelSpec::default(),
            })
            .expect("first search");
            let second = run_search(&SearchRequest {
                engine: Engine::Bm25,
                query: "agent memory".to_string(),
                path: corpus.path().to_path_buf(),
                limit: 10,
                shortlist: 10,
                dense_model: DenseModelSpec::default(),
            })
            .expect("second search");

            assert_eq!(first.indexed_files, 3);
            assert_eq!(first.skipped_files, 1);
            assert_eq!(first.results, second.results);
        }
    }

    mod rich_document {
        use super::*;
        use crate::extract::SourceKind;

        mod extractor_boundary {
            use super::*;

            #[test]
            fn routes_text_and_html_documents_through_shared_extractor() {
                let corpus = sample_rich_search_tree();
                let loaded = load_search_corpus(corpus.path()).expect("loaded corpus");

                assert_eq!(loaded.indexed_files, 2);
                assert_eq!(loaded.skipped_files, 1);

                let html = loaded
                    .documents
                    .iter()
                    .find(|document| document.path.ends_with("docs/service.html"))
                    .expect("html document");
                assert_eq!(html.source_kind, SourceKind::Html);
                assert!(html.text().contains("HTML Heading"));
                assert!(html.text().contains("Service Catalog"));

                let text = loaded
                    .documents
                    .iter()
                    .find(|document| document.path.ends_with("notes.txt"))
                    .expect("text document");
                assert_eq!(text.source_kind, SourceKind::Text);
                assert!(text.text().contains("service catalog"));
            }
        }

        mod segments {
            use super::*;

            #[test]
            fn segment_identity_is_stable_for_supported_documents() {
                let corpus = supported_fixture_tree();

                let first = load_search_corpus(corpus.path()).expect("first corpus load");
                let second = load_search_corpus(corpus.path()).expect("second corpus load");

                assert_eq!(first.indexed_files, 6);
                assert_eq!(second.indexed_files, 6);

                let first_segments = first
                    .documents
                    .iter()
                    .map(|document| {
                        assert!(
                            !document.segments().is_empty(),
                            "{} should emit at least one segment",
                            document.path.display()
                        );
                        (
                            document.path.display().to_string(),
                            document.id.clone(),
                            document
                                .segments()
                                .iter()
                                .map(|segment| segment.id.clone())
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>();
                let second_segments = second
                    .documents
                    .iter()
                    .map(|document| {
                        (
                            document.path.display().to_string(),
                            document.id.clone(),
                            document
                                .segments()
                                .iter()
                                .map(|segment| segment.id.clone())
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>();

                assert_eq!(first_segments, second_segments);
            }

            #[test]
            fn structure_aware_segments_are_source_aware() {
                let corpus = supported_fixture_tree();
                let loaded = load_search_corpus(corpus.path()).expect("loaded corpus");

                let html = loaded
                    .documents
                    .iter()
                    .find(|document| document.path.ends_with("docs/service.html"))
                    .expect("html document");
                assert!(
                    html.segments()
                        .iter()
                        .any(|segment| segment.label.contains("HTML Heading"))
                );

                let pdf = loaded
                    .documents
                    .iter()
                    .find(|document| document.path.ends_with("docs/architecture.pdf"))
                    .expect("pdf document");
                assert!(pdf.segments()[0].label.starts_with("page "));

                let docx = loaded
                    .documents
                    .iter()
                    .find(|document| document.path.ends_with("docs/roadmap-docx.docx"))
                    .expect("docx document");
                assert!(docx.segments()[0].label.starts_with("section "));

                let xlsx = loaded
                    .documents
                    .iter()
                    .find(|document| document.path.ends_with("docs/roadmap-sheet.xlsx"))
                    .expect("xlsx document");
                assert!(xlsx.segments()[0].label.starts_with("sheet "));

                let pptx = loaded
                    .documents
                    .iter()
                    .find(|document| document.path.ends_with("docs/roadmap-slides.pptx"))
                    .expect("pptx document");
                assert!(pptx.segments()[0].label.starts_with("slide "));
            }

            #[test]
            fn segment_text_preservation_keeps_section_local_text() {
                let corpus = supported_fixture_tree();
                let loaded = load_search_corpus(corpus.path()).expect("loaded corpus");

                let html = loaded
                    .documents
                    .iter()
                    .find(|document| document.path.ends_with("docs/service.html"))
                    .expect("html document");
                assert!(html.segments().iter().any(|segment| {
                    segment
                        .text
                        .contains("Service Catalog for the agent platform.")
                }));

                let text = loaded
                    .documents
                    .iter()
                    .find(|document| document.path.ends_with("notes.txt"))
                    .expect("text document");
                assert!(text.segments().iter().any(|segment| {
                    segment
                        .text
                        .contains("Plain text fallback for the service catalog.")
                }));

                let pdf = loaded
                    .documents
                    .iter()
                    .find(|document| document.path.ends_with("docs/architecture.pdf"))
                    .expect("pdf document");
                assert!(pdf.segments().iter().any(|segment| {
                    segment
                        .text
                        .to_lowercase()
                        .contains("architecture decision")
                }));

                let docx = loaded
                    .documents
                    .iter()
                    .find(|document| document.path.ends_with("docs/roadmap-docx.docx"))
                    .expect("docx document");
                assert!(
                    docx.segments()
                        .iter()
                        .any(|segment| segment.text.to_lowercase().contains("quarterly roadmap"))
                );

                let xlsx = loaded
                    .documents
                    .iter()
                    .find(|document| document.path.ends_with("docs/roadmap-sheet.xlsx"))
                    .expect("xlsx document");
                assert!(
                    xlsx.segments()
                        .iter()
                        .any(|segment| segment.text.to_lowercase().contains("quarterly roadmap"))
                );

                let pptx = loaded
                    .documents
                    .iter()
                    .find(|document| document.path.ends_with("docs/roadmap-slides.pptx"))
                    .expect("pptx document");
                assert!(
                    pptx.segments()
                        .iter()
                        .any(|segment| segment.text.to_lowercase().contains("quarterly roadmap"))
                );
            }
        }

        mod html {
            use super::*;

            #[test]
            fn html_files_are_searchable_without_preprocessing() {
                let corpus = sample_rich_search_tree();
                let response = run_search(&SearchRequest {
                    engine: Engine::Bm25,
                    query: "html heading".to_string(),
                    path: corpus.path().to_path_buf(),
                    limit: 10,
                    shortlist: 10,
                    dense_model: DenseModelSpec::default(),
                })
                .expect("search response");

                assert_eq!(response.results[0].rank, 1);
                assert!(response.results[0].path.ends_with("docs/service.html"));
                assert!(response.results[0].snippet.contains("HTML Heading"));
            }
        }

        mod pdf {
            use std::path::Path;

            use super::*;

            #[test]
            fn pdf_files_are_searchable_without_external_conversion() {
                let fixture_root =
                    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rich-docs");
                let response = run_search(&SearchRequest {
                    engine: Engine::Bm25,
                    query: "architecture decision".to_string(),
                    path: fixture_root,
                    limit: 10,
                    shortlist: 10,
                    dense_model: DenseModelSpec::default(),
                })
                .expect("search response");

                assert_eq!(response.results[0].rank, 1);
                assert!(response.results[0].path.ends_with("docs/architecture.pdf"));
                assert!(
                    response.results[0]
                        .snippet
                        .to_lowercase()
                        .contains("architecture decision")
                );
            }
        }

        mod office {
            use std::path::Path;

            use super::*;

            #[test]
            fn office_documents_are_searchable_without_external_conversion() {
                let fixture_root =
                    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rich-docs");
                let response = run_search(&SearchRequest {
                    engine: Engine::Bm25,
                    query: "quarterly roadmap".to_string(),
                    path: fixture_root,
                    limit: 10,
                    shortlist: 10,
                    dense_model: DenseModelSpec::default(),
                })
                .expect("search response");

                let paths = response
                    .results
                    .iter()
                    .map(|hit| hit.path.as_str())
                    .collect::<Vec<_>>();

                assert!(
                    paths
                        .iter()
                        .any(|path| path.ends_with("docs/roadmap-docx.docx"))
                );
                assert!(
                    paths
                        .iter()
                        .any(|path| path.ends_with("docs/roadmap-sheet.xlsx"))
                );
                assert!(
                    paths
                        .iter()
                        .any(|path| path.ends_with("docs/roadmap-slides.pptx"))
                );
            }
        }

        mod determinism {
            use std::path::Path;

            use super::*;

            #[test]
            fn mixed_format_search_results_are_deterministic() {
                let fixture_root =
                    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rich-docs");

                let first = run_search(&SearchRequest {
                    engine: Engine::Bm25,
                    query: "quarterly roadmap".to_string(),
                    path: fixture_root.clone(),
                    limit: 10,
                    shortlist: 10,
                    dense_model: DenseModelSpec::default(),
                })
                .expect("first search");
                let second = run_search(&SearchRequest {
                    engine: Engine::Bm25,
                    query: "quarterly roadmap".to_string(),
                    path: fixture_root,
                    limit: 10,
                    shortlist: 10,
                    dense_model: DenseModelSpec::default(),
                })
                .expect("second search");

                assert_eq!(first.indexed_files, second.indexed_files);
                assert_eq!(first.skipped_files, second.skipped_files);
                assert_eq!(first.results, second.results);
            }
        }

        mod skip_handling {
            use super::*;

            #[test]
            fn invalid_binary_files_are_skipped_deterministically() {
                let corpus = sample_rich_search_tree();

                let first = run_search(&SearchRequest {
                    engine: Engine::Bm25,
                    query: "service catalog".to_string(),
                    path: corpus.path().to_path_buf(),
                    limit: 10,
                    shortlist: 10,
                    dense_model: DenseModelSpec::default(),
                })
                .expect("first search");
                let second = run_search(&SearchRequest {
                    engine: Engine::Bm25,
                    query: "service catalog".to_string(),
                    path: corpus.path().to_path_buf(),
                    limit: 10,
                    shortlist: 10,
                    dense_model: DenseModelSpec::default(),
                })
                .expect("second search");

                assert_eq!(first.indexed_files, 2);
                assert_eq!(first.skipped_files, 1);
                assert_eq!(first.results, second.results);
            }
        }
    }

    fn sample_search_tree() -> tempfile::TempDir {
        let dir = tempdir().expect("search dir");
        stdfs::create_dir_all(dir.path().join("nested")).expect("nested dir");
        stdfs::write(
            dir.path().join("nested/alpha.txt"),
            "Retrieval architecture guide\n\nBM25 makes retrieval architecture explainable.",
        )
        .expect("write alpha");
        stdfs::write(
            dir.path().join("notes.md"),
            "Agent memory note\n\nUseful for semantic follow-up later.",
        )
        .expect("write notes");
        stdfs::write(
            dir.path().join("nested/other.rs"),
            "fn main() { println!(\"retrieval architecture in code\"); }",
        )
        .expect("write other");
        stdfs::write(dir.path().join("blob.bin"), [0xFF, 0xFE, 0xFD]).expect("write invalid utf8");
        dir
    }

    fn sample_rich_search_tree() -> tempfile::TempDir {
        let dir = tempdir().expect("rich search dir");
        stdfs::create_dir_all(dir.path().join("docs")).expect("docs dir");
        stdfs::write(
            dir.path().join("docs/service.html"),
            r#"<!doctype html>
<html>
  <body>
    <h1>HTML Heading</h1>
    <p>Service Catalog for the agent platform.</p>
  </body>
</html>
"#,
        )
        .expect("write html");
        stdfs::write(
            dir.path().join("notes.txt"),
            "service catalog note\n\nPlain text fallback for the service catalog.",
        )
        .expect("write notes");
        stdfs::write(dir.path().join("blob.bin"), [0xFF, 0xFE, 0xFD]).expect("write invalid blob");
        dir
    }

    fn supported_fixture_tree() -> tempfile::TempDir {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rich-docs");
        let dir = tempdir().expect("supported fixture dir");
        stdfs::create_dir_all(dir.path().join("docs")).expect("docs dir");

        for file in [
            "architecture.pdf",
            "roadmap-docx.docx",
            "roadmap-sheet.xlsx",
            "roadmap-slides.pptx",
            "service.html",
        ] {
            stdfs::copy(
                fixture_root.join("docs").join(file),
                dir.path().join("docs").join(file),
            )
            .expect("copy rich fixture");
        }
        stdfs::copy(fixture_root.join("notes.txt"), dir.path().join("notes.txt"))
            .expect("copy notes fixture");

        dir
    }
}
