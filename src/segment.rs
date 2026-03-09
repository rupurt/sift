use std::path::{Path, PathBuf};

use crate::extract::SourceKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub id: String,
    pub doc_id: String,
    pub path: PathBuf,
    pub source_kind: SourceKind,
    pub ordinal: usize,
    pub label: String,
    pub text: String,
}

#[derive(Debug, Clone)]
struct SegmentSeed {
    label: String,
    text: String,
}

pub fn build_segments(
    document_id: &str,
    path: &Path,
    source_kind: SourceKind,
    text: &str,
) -> Vec<Segment> {
    let seeds = match source_kind {
        SourceKind::Pdf => build_pdf_segments(text),
        SourceKind::Pptx => build_block_segments(text, "slide"),
        SourceKind::Xlsx => build_block_segments(text, "sheet"),
        SourceKind::Text | SourceKind::Html | SourceKind::Docx => {
            build_block_segments(text, "section")
        }
    };
    let seeds = if seeds.is_empty() {
        vec![SegmentSeed {
            label: default_label(source_kind, 1),
            text: normalize_text(text),
        }]
    } else {
        seeds
    };

    seeds
        .into_iter()
        .enumerate()
        .map(|(index, seed)| Segment {
            id: format!("{document_id}::segment:{:04}", index + 1),
            doc_id: document_id.to_string(),
            path: path.to_path_buf(),
            source_kind,
            ordinal: index + 1,
            label: seed.label,
            text: seed.text,
        })
        .collect()
}

fn build_pdf_segments(text: &str) -> Vec<SegmentSeed> {
    let pages = text
        .split('\u{000C}')
        .map(normalize_text)
        .filter(|page| !page.is_empty())
        .collect::<Vec<_>>();
    if !pages.is_empty() {
        return pages
            .into_iter()
            .enumerate()
            .map(|(index, page)| SegmentSeed {
                label: label_with_heading("page", index + 1, heading_from_text(&page).as_deref()),
                text: page,
            })
            .collect();
    }

    build_block_segments(text, "page")
}

fn build_block_segments(text: &str, prefix: &str) -> Vec<SegmentSeed> {
    let blocks = split_blocks(text);
    if blocks.is_empty() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut index = 0;

    while index < blocks.len() {
        let block = &blocks[index];
        let ordinal = segments.len() + 1;

        if looks_like_heading(block) && index + 1 < blocks.len() {
            let heading = clean_heading(block);
            let body = normalize_text(&blocks[index + 1]);
            let text = if body.is_empty() {
                heading.clone()
            } else {
                format!("{heading}\n\n{body}")
            };
            segments.push(SegmentSeed {
                label: label_with_heading(prefix, ordinal, Some(&heading)),
                text,
            });
            index += 2;
            continue;
        }

        let normalized = normalize_text(block);
        if normalized.is_empty() {
            index += 1;
            continue;
        }

        segments.push(SegmentSeed {
            label: label_with_heading(prefix, ordinal, heading_from_text(&normalized).as_deref()),
            text: normalized,
        });
        index += 1;
    }

    segments
}

fn split_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();

    for line in text.replace("\r\n", "\n").replace('\r', "\n").lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                blocks.push(current.join("\n"));
                current.clear();
            }
            continue;
        }
        current.push(line.trim_end().to_string());
    }

    if !current.is_empty() {
        blocks.push(current.join("\n"));
    }

    blocks
}

fn normalize_text(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim()
        .to_string()
}

fn looks_like_heading(block: &str) -> bool {
    let trimmed = block.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.starts_with('#') {
        return true;
    }

    let lines = trimmed.lines().collect::<Vec<_>>();
    if lines.len() != 1 {
        return false;
    }

    let line = lines[0].trim();
    line.len() <= 80
        && !line.ends_with('.')
        && !line.ends_with('!')
        && !line.ends_with('?')
        && line.chars().any(|ch| ch.is_alphabetic())
}

fn heading_from_text(text: &str) -> Option<String> {
    let line = text.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }
    if looks_like_heading(line) {
        Some(clean_heading(line))
    } else {
        None
    }
}

fn clean_heading(text: &str) -> String {
    text.trim()
        .trim_start_matches('#')
        .trim()
        .replace('\n', " ")
}

fn label_with_heading(prefix: &str, ordinal: usize, heading: Option<&str>) -> String {
    match heading {
        Some(heading) if !heading.is_empty() => format!("{prefix} {ordinal}: {heading}"),
        _ => format!("{prefix} {ordinal}"),
    }
}

fn default_label(source_kind: SourceKind, ordinal: usize) -> String {
    match source_kind {
        SourceKind::Pdf => label_with_heading("page", ordinal, None),
        SourceKind::Pptx => label_with_heading("slide", ordinal, None),
        SourceKind::Xlsx => label_with_heading("sheet", ordinal, None),
        SourceKind::Text | SourceKind::Html | SourceKind::Docx => {
            label_with_heading("section", ordinal, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{Segment, build_segments};
    use crate::extract::SourceKind;

    #[test]
    fn merges_standalone_heading_blocks_into_section_segments() {
        let segments = build_segments(
            "doc",
            Path::new("service.html"),
            SourceKind::Html,
            "# HTML Heading\n\nService Catalog for the agent platform.",
        );

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].label, "section 1: HTML Heading");
        assert!(segments[0].text.contains("Service Catalog"));
    }

    #[test]
    fn produces_stable_segment_identifiers() {
        let first = build_segments(
            "doc-1",
            Path::new("notes.txt"),
            SourceKind::Text,
            "Quarterly roadmap\n\nFollow-up details.",
        );
        let second = build_segments(
            "doc-1",
            Path::new("notes.txt"),
            SourceKind::Text,
            "Quarterly roadmap\n\nFollow-up details.",
        );

        assert_eq!(
            first.iter().map(segment_id).collect::<Vec<_>>(),
            second.iter().map(segment_id).collect::<Vec<_>>()
        );
    }

    fn segment_id(segment: &Segment) -> String {
        segment.id.clone()
    }
}
