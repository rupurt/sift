use std::cmp::Ordering;
use std::path::Path;

use anyhow::Result;
use strsim::{jaro_winkler, normalized_levenshtein};

use crate::search::domain::{
    Candidate, CandidateList, ContextArtifact, ContributorScore, PreparedCorpus, QueryVariant,
    Retriever, RetrieverPolicy, tokenize,
};

pub struct PathFuzzyRetriever;

impl Retriever for PathFuzzyRetriever {
    fn retrieve(
        &self,
        query: &[QueryVariant],
        corpus: &PreparedCorpus,
        limit: usize,
        _verbose: u8,
    ) -> Result<CandidateList> {
        let mut results = corpus
            .artifacts
            .iter()
            .filter_map(|artifact| {
                let score = best_variant_score(query, |structural_query| {
                    score_path_match(structural_query, &artifact.path)
                });

                (score >= 0.78).then(|| Candidate {
                    id: artifact.id.clone(),
                    path: artifact.path.clone(),
                    score,
                    contributors: vec![ContributorScore {
                        retriever: RetrieverPolicy::PathFuzzy,
                        score,
                    }],
                    snippet: None,
                    snippet_location: None,
                })
            })
            .collect::<Vec<_>>();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        results.truncate(limit);

        Ok(CandidateList { results })
    }

    fn policy(&self) -> RetrieverPolicy {
        RetrieverPolicy::PathFuzzy
    }
}

pub struct SegmentFuzzyRetriever;

impl Retriever for SegmentFuzzyRetriever {
    fn retrieve(
        &self,
        query: &[QueryVariant],
        corpus: &PreparedCorpus,
        limit: usize,
        _verbose: u8,
    ) -> Result<CandidateList> {
        let mut results = corpus
            .artifacts
            .iter()
            .filter_map(|artifact| {
                best_segment_match(query, artifact).map(|segment_match| Candidate {
                    id: artifact.id.clone(),
                    path: artifact.path.clone(),
                    score: segment_match.score,
                    contributors: vec![ContributorScore {
                        retriever: RetrieverPolicy::SegmentFuzzy,
                        score: segment_match.score,
                    }],
                    snippet: Some(segment_match.snippet),
                    snippet_location: Some(segment_match.location),
                })
            })
            .collect::<Vec<_>>();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        results.truncate(limit);

        Ok(CandidateList { results })
    }

    fn policy(&self) -> RetrieverPolicy {
        RetrieverPolicy::SegmentFuzzy
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StructuralQuery {
    lower: String,
    compact: String,
    terms: Vec<String>,
    looks_path_like: bool,
    looks_symbol_like: bool,
}

#[derive(Debug, Clone)]
struct BestSegmentMatch {
    score: f64,
    snippet: String,
    location: String,
}

pub(crate) fn structural_rerank_bonus(query: &str, candidate: &Candidate) -> f64 {
    let structural_query = StructuralQuery::new(query);

    let path_bonus =
        normalize_signal(score_path_match(&structural_query, &candidate.path), 3.0) * 0.06;
    let heading_bonus = candidate
        .snippet_location
        .as_deref()
        .map(|heading| normalize_signal(score_text_match(&structural_query, heading), 2.0) * 0.035)
        .unwrap_or(0.0);
    let snippet_bonus = candidate
        .snippet
        .as_deref()
        .map(|snippet| normalize_signal(score_text_match(&structural_query, snippet), 2.0) * 0.02)
        .unwrap_or(0.0);
    let definition_bonus = candidate
        .snippet
        .as_deref()
        .map(|snippet| definition_alignment_bonus(&structural_query, snippet) * 0.08)
        .unwrap_or(0.0);

    (path_bonus + heading_bonus + snippet_bonus + definition_bonus).min(0.12)
}

fn best_variant_score<F>(query: &[QueryVariant], mut scorer: F) -> f64
where
    F: FnMut(&StructuralQuery) -> f64,
{
    query
        .iter()
        .filter(|variant| variant.weight > 0.0)
        .map(|variant| scorer(&StructuralQuery::new(&variant.text)) * variant.weight)
        .fold(0.0, f64::max)
}

fn best_segment_match(
    query: &[QueryVariant],
    artifact: &ContextArtifact,
) -> Option<BestSegmentMatch> {
    let mut best_match = None;
    let mut best_score = 0.0;

    for variant in query {
        if variant.weight <= 0.0 {
            continue;
        }

        let structural_query = StructuralQuery::new(&variant.text);
        for segment in artifact.segments() {
            let Some(mut segment_match) = score_segment_candidate(&structural_query, segment)
            else {
                continue;
            };
            segment_match.score *= variant.weight;

            if segment_match.score > best_score {
                best_score = segment_match.score;
                best_match = Some(segment_match);
            }
        }
    }

    best_match.filter(|segment_match| segment_match.score >= 0.78)
}

fn score_segment_candidate(
    query: &StructuralQuery,
    segment: &crate::segment::Segment,
) -> Option<BestSegmentMatch> {
    let label_score = score_text_match(query, &segment.label) * 0.45;
    let mut best_score = 0.0;
    let mut best_snippet = None;

    for line in segment
        .text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let line_score = score_text_match(query, line);
        if line_score <= 0.0 {
            continue;
        }

        let total = line_score + label_score + definition_alignment_bonus(query, line);
        if total > best_score {
            best_score = total;
            best_snippet = Some(truncate_snippet(line, 220));
        }
    }

    if best_snippet.is_none() {
        let segment_score = score_text_match(query, &segment.text);
        if segment_score > 0.0 {
            best_score =
                segment_score + label_score + definition_alignment_bonus(query, &segment.text);
            best_snippet = Some(truncate_snippet(&segment.text, 220));
        }
    }

    best_snippet.map(|snippet| BestSegmentMatch {
        score: best_score,
        snippet,
        location: segment.label.clone(),
    })
}

pub(crate) fn score_path_match(query: &StructuralQuery, path: &Path) -> f64 {
    let full_path = path.to_string_lossy();
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    let file_stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut best = score_component_match(query, full_path.as_ref())
        * if query.looks_path_like { 1.1 } else { 0.9 };
    best = best.max(score_component_match(query, &file_name) * 1.3);
    best = best.max(score_component_match(query, &file_stem) * 1.45);

    for component in path.components() {
        let component_text = component.as_os_str().to_string_lossy();
        best = best.max(score_component_match(query, component_text.as_ref()) * 1.1);
    }

    if best == 0.0 {
        return 0.0;
    }

    let lower_path = lowercase_text(full_path.as_ref());
    let overlap = query
        .terms
        .iter()
        .filter(|term| lower_path.contains(term.as_str()))
        .count() as f64
        * 0.06;

    best + overlap
}

fn score_text_match(query: &StructuralQuery, text: &str) -> f64 {
    let direct = score_component_match(query, text);
    let windowed = best_window_score(query, text);
    let lower = lowercase_text(text);
    let exact_bonus = if !query.lower.is_empty() && lower.contains(&query.lower) {
        0.35
    } else {
        0.0
    };

    direct.max(windowed) + exact_bonus
}

fn best_window_score(query: &StructuralQuery, text: &str) -> f64 {
    let tokens = tokenize(text);
    if tokens.is_empty() {
        return 0.0;
    }

    let min_window = query.terms.len().saturating_sub(1).max(1);
    let max_window = (query.terms.len() + 2).min(tokens.len()).min(6);
    if max_window < min_window {
        return 0.0;
    }

    let mut best = 0.0_f64;
    for window_size in min_window..=max_window {
        for window in tokens.windows(window_size) {
            let candidate = window.join(" ");
            best = best.max(score_component_match(query, &candidate));
        }
    }

    best
}

fn score_component_match(query: &StructuralQuery, candidate: &str) -> f64 {
    let candidate_lower = lowercase_text(candidate);
    let candidate_compact = compact_text(candidate);
    if query.compact.is_empty() || candidate_compact.is_empty() {
        return 0.0;
    }

    let base = subsequence_score(&query.compact, &candidate_lower)
        .max(normalized_levenshtein(&query.compact, &candidate_compact) * 0.95)
        .max(jaro_winkler(&query.compact, &candidate_compact) * 0.9);
    let contains_bonus =
        if candidate_lower.contains(&query.lower) || candidate_compact.contains(&query.compact) {
            0.55
        } else {
            0.0
        };
    let prefix_bonus = if candidate_compact.starts_with(&query.compact) {
        0.45
    } else {
        0.0
    };
    let overlap_bonus = token_overlap_bonus(query, &candidate_lower);

    if base < 0.5 && contains_bonus == 0.0 && prefix_bonus == 0.0 && overlap_bonus < 0.16 {
        return 0.0;
    }

    base + contains_bonus + prefix_bonus + overlap_bonus
}

fn token_overlap_bonus(query: &StructuralQuery, candidate_lower: &str) -> f64 {
    query
        .terms
        .iter()
        .filter(|term| term.len() >= 2 && candidate_lower.contains(term.as_str()))
        .count() as f64
        * 0.08
}

fn subsequence_score(query_compact: &str, candidate_lower: &str) -> f64 {
    let query_chars = query_compact.chars().collect::<Vec<_>>();
    if query_chars.is_empty() {
        return 0.0;
    }

    let candidate_chars = candidate_lower.chars().collect::<Vec<_>>();
    let mut query_index = 0;
    let mut first_match = None;
    let mut last_match = 0;
    let mut previous_match = None;
    let mut raw_score = 0.0;

    for (index, ch) in candidate_chars.iter().enumerate() {
        if query_index == query_chars.len() {
            break;
        }
        if !ch.is_alphanumeric() {
            continue;
        }
        if *ch != query_chars[query_index] {
            continue;
        }

        raw_score += 1.0;
        if is_boundary(&candidate_chars, index) {
            raw_score += 0.3;
        }
        if let Some(previous) = previous_match
            && index == previous + 1
        {
            raw_score += 0.35;
        }

        first_match.get_or_insert(index);
        last_match = index;
        previous_match = Some(index);
        query_index += 1;
    }

    if query_index != query_chars.len() {
        return 0.0;
    }

    let Some(first_match) = first_match else {
        return 0.0;
    };
    let span = last_match.saturating_sub(first_match) + 1;
    if span == 0 {
        return 0.0;
    }

    let density = query_chars.len() as f64 / span as f64;
    let normalized = raw_score / query_chars.len() as f64;
    normalized * (0.5 + density)
}

fn is_boundary(candidate_chars: &[char], index: usize) -> bool {
    if index == 0 {
        return true;
    }
    !candidate_chars[index - 1].is_alphanumeric()
}

fn definition_alignment_bonus(query: &StructuralQuery, text: &str) -> f64 {
    if !looks_like_definition(text) {
        return 0.0;
    }

    let lower = lowercase_text(text);
    let overlap = token_overlap_bonus(query, &lower).min(0.16);
    let symbol_bonus = if query.looks_symbol_like {
        normalize_signal(score_text_match(query, text), 2.0) * 0.18
    } else {
        0.0
    };

    0.16 + overlap + symbol_bonus
}

fn looks_like_definition(text: &str) -> bool {
    let lower = lowercase_text(text.trim_start());
    [
        "fn ",
        "pub fn ",
        "pub(crate) fn ",
        "struct ",
        "pub struct ",
        "enum ",
        "pub enum ",
        "trait ",
        "impl ",
        "mod ",
        "type ",
        "const ",
        "static ",
        "let ",
        "function ",
        "class ",
        "interface ",
        "def ",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

fn truncate_snippet(text: &str, max_chars: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let collapsed_chars = collapsed.chars().collect::<Vec<_>>();
    if collapsed_chars.len() <= max_chars {
        return collapsed;
    }

    collapsed_chars
        .into_iter()
        .take(max_chars.saturating_sub(3))
        .collect::<String>()
        + "..."
}

fn normalize_signal(score: f64, ceiling: f64) -> f64 {
    if !score.is_finite() || ceiling <= 0.0 {
        return 0.0;
    }
    (score / ceiling).clamp(0.0, 1.0)
}

fn lowercase_text(text: &str) -> String {
    text.chars().flat_map(|ch| ch.to_lowercase()).collect()
}

fn compact_text(text: &str) -> String {
    lowercase_text(text)
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .collect()
}

impl StructuralQuery {
    fn new(query: &str) -> Self {
        let trimmed = query.trim();
        let without_intent = trimmed.split("Intent:").next().unwrap_or(trimmed).trim();
        let raw = without_intent.trim_end_matches('.').trim().to_string();
        let lower = lowercase_text(&raw);
        let compact = compact_text(&raw);
        let terms = tokenize(&raw);
        let looks_path_like = raw.contains('/')
            || raw.contains('\\')
            || raw.contains('.')
            || raw.contains('_')
            || raw.contains('-');
        let looks_symbol_like =
            raw.contains("::") || raw.contains('_') || raw.chars().any(|ch| ch.is_uppercase());

        Self {
            lower,
            compact,
            terms,
            looks_path_like,
            looks_symbol_like,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::extract::SourceKind;
    use crate::search::adapters::PositionAwareReranker;
    use crate::search::domain::{
        AcquisitionAdapterKind, ArtifactBudget, ArtifactFreshness, ArtifactProvenance,
        ContextArtifactKind, Reranker,
    };
    use crate::segment::Segment;

    #[test]
    fn path_fuzzy_retriever_prefers_filename_like_queries() {
        let retriever = PathFuzzyRetriever;
        let corpus = sample_corpus();
        let query = vec![QueryVariant {
            text: "sift_request_factry".to_string(),
            weight: 1.0,
        }];

        let results = retriever
            .retrieve(&query, &corpus, 5, 0)
            .expect("path retrieval");

        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].id, "doc-path");
    }

    #[test]
    fn segment_fuzzy_retriever_returns_snippet_evidence() {
        let retriever = SegmentFuzzyRetriever;
        let corpus = sample_corpus();
        let query = vec![QueryVariant {
            text: "search plan fir".to_string(),
            weight: 1.0,
        }];

        let results = retriever
            .retrieve(&query, &corpus, 5, 0)
            .expect("segment fuzzy retrieval");

        assert_eq!(results.results[0].id, "doc-path");
        assert_eq!(
            results.results[0].snippet.as_deref(),
            Some("pub(crate) fn search_plan_for(strategy: RetrievalStrategy) -> SearchPlan {")
        );
        assert_eq!(
            results.results[0].snippet_location.as_deref(),
            Some("section 1: Search Helpers")
        );
    }

    #[test]
    fn position_aware_reranker_boosts_structural_matches() {
        let reranker = PositionAwareReranker;
        let candidates = CandidateList {
            results: vec![
                Candidate {
                    id: "weak-structure".to_string(),
                    path: "notes/random.md".into(),
                    score: 0.05,
                    contributors: vec![],
                    snippet: Some("unrelated notes".to_string()),
                    snippet_location: Some("section 1".to_string()),
                },
                Candidate {
                    id: "strong-structure".to_string(),
                    path: "docs/reactor_architecture.md".into(),
                    score: 0.04,
                    contributors: vec![],
                    snippet: Some("## Reactor Architecture".to_string()),
                    snippet_location: Some("section 2: Reactor Architecture".to_string()),
                },
            ],
        };

        let reranked = reranker
            .rerank("reactor architecture", candidates, 2)
            .expect("reranking");

        assert_eq!(reranked.results[0].id, "strong-structure");
    }

    fn sample_corpus() -> PreparedCorpus<'static> {
        let artifacts = vec![
            ContextArtifact {
                id: "doc-path".to_string(),
                kind: ContextArtifactKind::File,
                path: "src/infrastructure/sift_request_factory.rs".into(),
                source_kind: SourceKind::Text,
                length: 120,
                terms: document_terms(
                    "Search Helpers\n\npub(crate) fn search_plan_for(strategy: RetrievalStrategy) -> SearchPlan {\n    SearchPlan::default_page_index_hybrid()\n}",
                ),
                text: "Search Helpers\n\npub(crate) fn search_plan_for(strategy: RetrievalStrategy) -> SearchPlan {\n    SearchPlan::default_page_index_hybrid()\n}".to_string(),
                segments: vec![Segment {
                    id: "doc-path::segment:0001".to_string(),
                    artifact_id: "doc-path".to_string(),
                    path: "src/infrastructure/sift_request_factory.rs".into(),
                    source_kind: SourceKind::Text,
                    ordinal: 1,
                    label: "section 1: Search Helpers".to_string(),
                    text: "Search Helpers\n\npub(crate) fn search_plan_for(strategy: RetrievalStrategy) -> SearchPlan {\n    SearchPlan::default_page_index_hybrid()\n}".to_string(),
                    embedding: None,
                }],
                provenance: ArtifactProvenance {
                    adapter: AcquisitionAdapterKind::FileSystem,
                    source: "src/infrastructure/sift_request_factory.rs".to_string(),
                    synthetic: false,
                },
                freshness: ArtifactFreshness {
                    observed_unix_secs: 0,
                    modified_unix_secs: None,
                },
                budget: ArtifactBudget::from_text("Search Helpers", 1),
            },
            ContextArtifact {
                id: "doc-other".to_string(),
                kind: ContextArtifactKind::File,
                path: "src/application/runner.rs".into(),
                source_kind: SourceKind::Text,
                length: 40,
                terms: document_terms("runner wiring and unrelated helpers"),
                text: "runner wiring and unrelated helpers".to_string(),
                segments: vec![Segment {
                    id: "doc-other::segment:0001".to_string(),
                    artifact_id: "doc-other".to_string(),
                    path: "src/application/runner.rs".into(),
                    source_kind: SourceKind::Text,
                    ordinal: 1,
                    label: "section 1".to_string(),
                    text: "runner wiring and unrelated helpers".to_string(),
                    embedding: None,
                }],
                provenance: ArtifactProvenance {
                    adapter: AcquisitionAdapterKind::FileSystem,
                    source: "src/application/runner.rs".to_string(),
                    synthetic: false,
                },
                freshness: ArtifactFreshness {
                    observed_unix_secs: 0,
                    modified_unix_secs: None,
                },
                budget: ArtifactBudget::from_text("runner wiring and unrelated helpers", 1),
            },
        ];

        let leaked = Box::leak(artifacts.into_boxed_slice());
        PreparedCorpus {
            artifacts: leaked,
            bm25_index: None,
        }
    }

    fn document_terms(text: &str) -> HashMap<String, usize> {
        crate::search::tokenize(text)
            .into_iter()
            .fold(HashMap::new(), |mut terms, term| {
                *terms.entry(term).or_insert(0) += 1;
                terms
            })
    }
}
