use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use serde::{Deserialize, Serialize};

const HTML_RENDER_WIDTH: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceKind {
    Text,
    Html,
    Pdf,
    Docx,
    Xlsx,
    Pptx,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedDocument {
    pub text: String,
    pub source_kind: SourceKind,
}

pub fn extract_path(path: &Path) -> Result<Option<ExtractedDocument>> {
    if is_html_path(path) {
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read document {}", path.display()))?;
        extract_html(&bytes)
    } else if is_pdf_path(path) {
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read document {}", path.display()))?;
        extract_pdf(&bytes)
    } else if let Some(source_kind) = office_source_kind(path) {
        extract_office(path, source_kind)
    } else {
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read document {}", path.display()))?;
        Ok(extract_utf8(&bytes))
    }
}

fn is_html_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("html" | "htm")
    )
}

fn is_pdf_path(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("pdf"))
}

fn office_source_kind(path: &Path) -> Option<SourceKind> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("docx") => Some(SourceKind::Docx),
        Some("xlsx") => Some(SourceKind::Xlsx),
        Some("pptx") => Some(SourceKind::Pptx),
        _ => None,
    }
}

fn extract_utf8(bytes: &[u8]) -> Option<ExtractedDocument> {
    let text = String::from_utf8(bytes.to_vec()).ok()?;

    Some(ExtractedDocument {
        text,
        source_kind: SourceKind::Text,
    })
}

fn extract_html(bytes: &[u8]) -> Result<Option<ExtractedDocument>> {
    let text = html2text::from_read(bytes, HTML_RENDER_WIDTH).context("render html as text")?;

    Ok(Some(ExtractedDocument {
        text,
        source_kind: SourceKind::Html,
    }))
}

fn extract_pdf(bytes: &[u8]) -> Result<Option<ExtractedDocument>> {
    let text = pdf_extract::extract_text_from_mem(bytes).context("extract pdf text")?;

    Ok(Some(ExtractedDocument {
        text,
        source_kind: SourceKind::Pdf,
    }))
}

fn extract_office(path: &Path, source_kind: SourceKind) -> Result<Option<ExtractedDocument>> {
    let text = undoc::extract_text(path)
        .with_context(|| format!("extract office text from {}", path.display()))?;

    Ok(Some(ExtractedDocument { text, source_kind }))
}
