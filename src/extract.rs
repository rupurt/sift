use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

const HTML_RENDER_WIDTH: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Text,
    Html,
    Pdf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedDocument {
    pub text: String,
    pub source_kind: SourceKind,
}

pub fn extract_path(path: &Path) -> Result<Option<ExtractedDocument>> {
    let bytes =
        fs::read(path).with_context(|| format!("failed to read document {}", path.display()))?;

    if is_html_path(path) {
        extract_html(&bytes)
    } else if is_pdf_path(path) {
        extract_pdf(&bytes)
    } else {
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
