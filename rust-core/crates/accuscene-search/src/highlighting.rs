//! Search result highlighting

use crate::error::SearchResult;
use crate::ranking::Highlight;

/// Highlighter for search results
pub struct Highlighter {
    pre_tag: String,
    post_tag: String,
    max_snippet_length: usize,
    max_snippets: usize,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            pre_tag: "<em>".to_string(),
            post_tag: "</em>".to_string(),
            max_snippet_length: 150,
            max_snippets: 3,
        }
    }

    pub fn with_tags(mut self, pre: String, post: String) -> Self {
        self.pre_tag = pre;
        self.post_tag = post;
        self
    }

    pub fn with_snippet_length(mut self, length: usize) -> Self {
        self.max_snippet_length = length;
        self
    }

    pub fn with_max_snippets(mut self, max: usize) -> Self {
        self.max_snippets = max;
        self
    }

    /// Highlight matches in text
    pub fn highlight(
        &self,
        text: &str,
        query_terms: &[String],
    ) -> SearchResult<Vec<Highlight>> {
        let mut highlights = Vec::new();
        let text_lower = text.to_lowercase();

        // Find all match positions
        let mut matches: Vec<(usize, usize, String)> = Vec::new();

        for term in query_terms {
            let term_lower = term.to_lowercase();
            let mut start = 0;

            while let Some(pos) = text_lower[start..].find(&term_lower) {
                let actual_pos = start + pos;
                matches.push((actual_pos, actual_pos + term.len(), term.clone()));
                start = actual_pos + term.len();
            }
        }

        if matches.is_empty() {
            return Ok(highlights);
        }

        // Sort matches by position
        matches.sort_by_key(|(start, _, _)| *start);

        // Create snippets around matches
        let snippets = self.create_snippets(text, &matches);

        for snippet in snippets.into_iter().take(self.max_snippets) {
            highlights.push(Highlight {
                field: "content".to_string(),
                snippet,
            });
        }

        Ok(highlights)
    }

    fn create_snippets(
        &self,
        text: &str,
        matches: &[(usize, usize, String)],
    ) -> Vec<String> {
        let mut snippets = Vec::new();
        let mut used_ranges: Vec<(usize, usize)> = Vec::new();

        for (start, end, term) in matches {
            // Skip if this match is already in a snippet
            if used_ranges.iter().any(|(s, e)| start >= s && start < e) {
                continue;
            }

            // Calculate snippet boundaries
            let snippet_start = start.saturating_sub(self.max_snippet_length / 2);
            let snippet_end = std::cmp::min(
                text.len(),
                end + self.max_snippet_length / 2,
            );

            // Adjust to word boundaries
            let snippet_start = self.find_word_boundary(text, snippet_start, true);
            let snippet_end = self.find_word_boundary(text, snippet_end, false);

            // Extract and highlight snippet
            let mut snippet = text[snippet_start..snippet_end].to_string();

            // Highlight all matches in this snippet
            for (match_start, match_end, match_term) in matches {
                if *match_start >= snippet_start && *match_end <= snippet_end {
                    let relative_start = match_start - snippet_start;
                    let relative_end = match_end - snippet_start;

                    let highlighted = format!(
                        "{}{}{}",
                        self.pre_tag,
                        &snippet[relative_start..relative_end],
                        self.post_tag
                    );

                    snippet = snippet[..relative_start].to_string()
                        + &highlighted
                        + &snippet[relative_end..];
                }
            }

            // Add ellipsis if needed
            if snippet_start > 0 {
                snippet = format!("...{}", snippet);
            }
            if snippet_end < text.len() {
                snippet = format!("{}...", snippet);
            }

            snippets.push(snippet);
            used_ranges.push((snippet_start, snippet_end));
        }

        snippets
    }

    fn find_word_boundary(&self, text: &str, pos: usize, search_backward: bool) -> usize {
        if pos >= text.len() {
            return text.len();
        }

        if pos == 0 {
            return 0;
        }

        let chars: Vec<char> = text.chars().collect();

        if search_backward {
            for i in (0..=pos).rev() {
                if i == 0 || chars[i].is_whitespace() {
                    return i;
                }
            }
            0
        } else {
            for i in pos..chars.len() {
                if chars[i].is_whitespace() {
                    return i;
                }
            }
            chars.len()
        }
    }

    /// Highlight with HTML-safe output
    pub fn highlight_html(
        &self,
        text: &str,
        query_terms: &[String],
    ) -> SearchResult<Vec<Highlight>> {
        // Escape HTML first
        let escaped_text = self.escape_html(text);

        // Then highlight
        self.highlight(&escaped_text, query_terms)
    }

    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Fast snippet extractor without highlighting
pub struct SnippetExtractor {
    max_length: usize,
}

impl SnippetExtractor {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }

    /// Extract a snippet around the first query match
    pub fn extract(&self, text: &str, query: &str) -> String {
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();

        if let Some(pos) = text_lower.find(&query_lower) {
            let start = pos.saturating_sub(self.max_length / 2);
            let end = std::cmp::min(text.len(), pos + query.len() + self.max_length / 2);

            let mut snippet = text[start..end].to_string();

            if start > 0 {
                snippet = format!("...{}", snippet);
            }
            if end < text.len() {
                snippet = format!("{}...", snippet);
            }

            snippet
        } else {
            // Return first N characters if no match
            let end = std::cmp::min(self.max_length, text.len());
            let mut snippet = text[..end].to_string();

            if end < text.len() {
                snippet = format!("{}...", snippet);
            }

            snippet
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlighter() {
        let highlighter = Highlighter::new();
        let text = "This is a test document with some test content for searching.";
        let query_terms = vec!["test".to_string()];

        let highlights = highlighter.highlight(text, &query_terms).unwrap();
        assert!(!highlights.is_empty());
        assert!(highlights[0].snippet.contains("<em>test</em>"));
    }

    #[test]
    fn test_highlighter_custom_tags() {
        let highlighter = Highlighter::new()
            .with_tags("<mark>".to_string(), "</mark>".to_string());

        let text = "Find this word in the text.";
        let query_terms = vec!["word".to_string()];

        let highlights = highlighter.highlight(text, &query_terms).unwrap();
        assert!(highlights[0].snippet.contains("<mark>word</mark>"));
    }

    #[test]
    fn test_snippet_extractor() {
        let extractor = SnippetExtractor::new(50);
        let text = "This is a very long document with lots of content. The important part is here in the middle. And then more content after.";
        let snippet = extractor.extract(text, "important");

        assert!(snippet.contains("important"));
        assert!(snippet.len() <= 60); // 50 + ellipsis
    }

    #[test]
    fn test_html_escape() {
        let highlighter = Highlighter::new();
        let escaped = highlighter.escape_html("<script>alert('xss')</script>");

        assert!(!escaped.contains("<script>"));
        assert!(escaped.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_multiple_matches() {
        let highlighter = Highlighter::new();
        let text = "The test is a test of the test system.";
        let query_terms = vec!["test".to_string()];

        let highlights = highlighter.highlight(text, &query_terms).unwrap();
        assert!(!highlights.is_empty());

        // Should highlight all occurrences of "test"
        let highlighted_count = highlights[0].snippet.matches("<em>").count();
        assert!(highlighted_count >= 3);
    }
}
