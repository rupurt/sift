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
    Png,
    Jpeg,
    Tiff,
    Bmp,
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
        extract_pdf(path, &bytes)
    } else if let Some(source_kind) = office_source_kind(path) {
        extract_office(path, source_kind)
    } else if let Some(source_kind) = image_source_kind(path) {
        extract_image(path, source_kind)
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

fn image_source_kind(path: &Path) -> Option<SourceKind> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("png") => Some(SourceKind::Png),
        Some("jpg" | "jpeg") => Some(SourceKind::Jpeg),
        Some("tiff" | "tif") => Some(SourceKind::Tiff),
        Some("bmp") => Some(SourceKind::Bmp),
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

fn is_scanned_pdf_heuristic(text: &str) -> bool {
    let alphanumeric_count = text.chars().filter(|c| c.is_alphanumeric()).count();
    alphanumeric_count < 50
}

fn extract_pdf(path: &Path, bytes: &[u8]) -> Result<Option<ExtractedDocument>> {
    #[cfg(feature = "ocr")]
    let mut text = pdf_extract::extract_text_from_mem(bytes).context("extract pdf text")?;
    #[cfg(not(feature = "ocr"))]
    let text = pdf_extract::extract_text_from_mem(bytes).context("extract pdf text")?;

    if is_scanned_pdf_heuristic(&text) {
        #[cfg(feature = "ocr")]
        {
            tracing::info!("PDF appears to be scanned, attempting OCR fallback: {}", path.display());
            match perform_ocr(path) {
                Ok(ocr_text) => {
                    text = ocr_text;
                }
                Err(e) => {
                    tracing::warn!("OCR fallback failed for PDF {}, returning sparse text: {}", path.display(), e);
                }
            }
        }
        #[cfg(not(feature = "ocr"))]
        {
            tracing::warn!("PDF appears to be scanned but OCR is disabled, returning sparse text: {}", path.display());
        }
    }

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

#[cfg(feature = "ocr")]
fn perform_ocr(path: &Path) -> Result<String> {
    let tesseract = tesseract_rs::TesseractAPI::new();
    tesseract
        .init(".", "eng")
        .map_err(|e| anyhow::anyhow!("failed to initialize Tesseract: {}", e))?;

    let text = tesseract
        .process_pages(
            path.to_str()
                .ok_or_else(|| anyhow::anyhow!("invalid file path for OCR"))?,
            None,
            0,
        )
        .map_err(|e| anyhow::anyhow!("failed to extract text via OCR: {}", e))?;

    Ok(text)
}

#[cfg(feature = "ocr")]
fn extract_image(path: &Path, source_kind: SourceKind) -> Result<Option<ExtractedDocument>> {
    let text = perform_ocr(path)?;
    Ok(Some(ExtractedDocument { text, source_kind }))
}

#[cfg(not(feature = "ocr"))]
fn extract_image(path: &Path, _source_kind: SourceKind) -> Result<Option<ExtractedDocument>> {
    tracing::warn!(
        "OCR is disabled; skipping image file: {}",
        path.display()
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_image_source_kind() {
        assert_eq!(image_source_kind(Path::new("test.png")), Some(SourceKind::Png));
        assert_eq!(image_source_kind(Path::new("test.jpg")), Some(SourceKind::Jpeg));
        assert_eq!(image_source_kind(Path::new("test.jpeg")), Some(SourceKind::Jpeg));
        assert_eq!(image_source_kind(Path::new("test.tiff")), Some(SourceKind::Tiff));
        assert_eq!(image_source_kind(Path::new("test.tif")), Some(SourceKind::Tiff));
        assert_eq!(image_source_kind(Path::new("test.bmp")), Some(SourceKind::Bmp));
        assert_eq!(image_source_kind(Path::new("test.txt")), None);
    }

    #[test]
    #[cfg(not(feature = "ocr"))]
    fn test_extract_path_skips_image_when_ocr_disabled() {
        // We don't need the file to exist because it should hit the image check first
        let result = extract_path(Path::new("test.png")).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_is_scanned_pdf_heuristic() {
        assert!(is_scanned_pdf_heuristic(""));
        assert!(is_scanned_pdf_heuristic("   \n \t  "));
        assert!(is_scanned_pdf_heuristic("1234567890")); // 10 chars
        assert!(!is_scanned_pdf_heuristic("This is a standard PDF with plenty of alphanumeric characters, clearly exceeding the threshold for the heuristic."));
    }
}
