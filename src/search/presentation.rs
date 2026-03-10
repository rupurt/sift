use super::domain::tokenize;

pub fn build_snippet(text: &str, query: &str) -> String {
    let query_terms = tokenize(query);
    if query_terms.is_empty() {
        return build_simple_snippet(text, 160);
    }

    let collapsed = text
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    let lowercase_text = collapsed.to_lowercase();

    // Find the first occurrence of any query term
    let mut first_match_pos = None;
    for term in &query_terms {
        if let Some(pos) = lowercase_text.find(term) {
            match first_match_pos {
                None => first_match_pos = Some(pos),
                Some(current) if pos < current => first_match_pos = Some(pos),
                _ => {}
            }
        }
    }

    let start_pos = first_match_pos.unwrap_or(0);
    // Move back a bit to get some context, but don't split words
    let mut context_start = 0;
    if start_pos > 40 {
        // Find a space around start_pos - 40 safely
        let target_pos = start_pos - 40;
        // Find the nearest char boundary at or before target_pos safely
        let boundary = collapsed
            .char_indices()
            .map(|(idx, _)| idx)
            .take_while(|&idx| idx <= target_pos)
            .last()
            .unwrap_or(0);

        context_start = collapsed[..boundary].rfind(' ').map(|p| p + 1).unwrap_or(0);
    }

    let limit = 240; // Show more content
    let mut snippet = collapsed
        .chars()
        .skip(collapsed[..context_start].chars().count())
        .take(limit)
        .collect::<String>();

    if context_start > 0 {
        snippet = format!("...{}", snippet);
    }
    if collapsed.chars().count() > context_start + limit {
        snippet.push_str("...");
    }

    highlight_matches(&snippet, &query_terms)
}

pub fn build_simple_snippet(text: &str, limit: usize) -> String {
    let collapsed = text
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    if collapsed.is_empty() {
        return String::new();
    }

    let mut snippet = collapsed.chars().take(limit).collect::<String>();
    if collapsed.chars().count() > limit {
        snippet.push_str("...");
    }
    snippet
}

fn highlight_matches(text: &str, terms: &[String]) -> String {
    if terms.is_empty() {
        return text.to_string();
    }

    let lowercase_text = text.to_lowercase();
    let mut highlights = Vec::new();

    let mut sorted_terms = terms.to_vec();
    sorted_terms.sort_by_key(|b| std::cmp::Reverse(b.len()));

    for term in sorted_terms {
        if term.is_empty() {
            continue;
        }
        for (pos, _) in lowercase_text.match_indices(&term) {
            let end = pos + term.len();
            // Check if this range overlaps with an existing highlight
            if !highlights.iter().any(|(h_start, h_end)| {
                (pos >= *h_start && pos < *h_end) || (end > *h_start && end <= *h_end)
            }) {
                highlights.push((pos, end));
            }
        }
    }

    // Sort highlights by start position descending to apply from back to front
    highlights.sort_by_key(|&(start, _)| std::cmp::Reverse(start));

    let mut highlighted = text.to_string();
    for (start, end) in highlights {
        highlighted.insert_str(end, "\x1b[0m");
        highlighted.insert_str(start, "\x1b[1;33m");
    }

    highlighted
}
