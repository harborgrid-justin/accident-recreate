//! Text size and readability analysis for WCAG compliance
//!
//! Implements WCAG 2.1 Success Criterion 1.4.4 (Resize Text)
//! and 1.4.12 (Text Spacing)

use crate::error::{A11yError, Result};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Minimum recommended font sizes (in pixels)
pub const MIN_FONT_SIZE_NORMAL: f32 = 16.0;
pub const MIN_FONT_SIZE_LARGE: f32 = 18.0;
pub const MIN_FONT_SIZE_HEADING: f32 = 24.0;

/// Text classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextClassification {
    /// Normal body text
    Normal,
    /// Large text (18pt+ or 14pt+ bold)
    Large,
    /// Heading text
    Heading,
    /// Label text
    Label,
    /// Code or monospace text
    Code,
}

/// Text spacing requirements (WCAG 2.1 1.4.12)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TextSpacing {
    /// Line height (should be at least 1.5 times font size)
    pub line_height: f32,
    /// Paragraph spacing (should be at least 2 times font size)
    pub paragraph_spacing: f32,
    /// Letter spacing (should be at least 0.12 times font size)
    pub letter_spacing: f32,
    /// Word spacing (should be at least 0.16 times font size)
    pub word_spacing: f32,
}

impl TextSpacing {
    /// Create spacing requirements for a given font size
    pub fn for_font_size(font_size: f32) -> Self {
        Self {
            line_height: font_size * 1.5,
            paragraph_spacing: font_size * 2.0,
            letter_spacing: font_size * 0.12,
            word_spacing: font_size * 0.16,
        }
    }

    /// Check if spacing meets WCAG requirements
    pub fn meets_wcag(&self, font_size: f32) -> bool {
        self.line_height >= font_size * 1.5
            && self.paragraph_spacing >= font_size * 2.0
            && self.letter_spacing >= font_size * 0.12
            && self.word_spacing >= font_size * 0.16
    }
}

/// Text properties for accessibility analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextProperties {
    pub font_size: f32,
    pub font_weight: u16,
    pub classification: TextClassification,
    pub spacing: TextSpacing,
    pub max_width: Option<f32>,
}

impl TextProperties {
    /// Create new text properties
    pub fn new(font_size: f32) -> Self {
        Self {
            font_size,
            font_weight: 400,
            classification: TextClassification::Normal,
            spacing: TextSpacing::for_font_size(font_size),
            max_width: None,
        }
    }

    /// Check if text is considered "large" by WCAG standards
    pub fn is_large_text(&self) -> bool {
        (self.font_size >= 18.0) || (self.font_size >= 14.0 && self.font_weight >= 700)
    }

    /// Get minimum recommended font size for classification
    pub fn min_recommended_size(&self) -> f32 {
        match self.classification {
            TextClassification::Normal => MIN_FONT_SIZE_NORMAL,
            TextClassification::Large => MIN_FONT_SIZE_LARGE,
            TextClassification::Heading => MIN_FONT_SIZE_HEADING,
            TextClassification::Label => MIN_FONT_SIZE_NORMAL,
            TextClassification::Code => MIN_FONT_SIZE_NORMAL,
        }
    }

    /// Check if font size meets minimum requirements
    pub fn meets_size_requirements(&self) -> bool {
        self.font_size >= self.min_recommended_size()
    }
}

/// Readability analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityScore {
    /// Average words per sentence
    pub avg_words_per_sentence: f32,
    /// Average syllables per word
    pub avg_syllables_per_word: f32,
    /// Flesch Reading Ease score (0-100, higher is easier)
    pub flesch_score: f32,
    /// Flesch-Kincaid Grade Level
    pub grade_level: f32,
    /// Total word count
    pub word_count: usize,
    /// Total sentence count
    pub sentence_count: usize,
}

impl ReadabilityScore {
    /// Get readability level description
    pub fn level_description(&self) -> &'static str {
        match self.flesch_score as i32 {
            90..=100 => "Very Easy (5th grade)",
            80..=89 => "Easy (6th grade)",
            70..=79 => "Fairly Easy (7th grade)",
            60..=69 => "Standard (8th-9th grade)",
            50..=59 => "Fairly Difficult (10th-12th grade)",
            30..=49 => "Difficult (College)",
            _ => "Very Difficult (College graduate)",
        }
    }

    /// Check if text is at recommended reading level (8th grade or below)
    pub fn is_accessible(&self) -> bool {
        self.flesch_score >= 60.0
    }
}

/// Text analyzer for accessibility checks
pub struct TextAnalyzer;

impl TextAnalyzer {
    /// Analyze text readability
    pub fn analyze_readability(text: &str) -> ReadabilityScore {
        let sentences = Self::count_sentences(text);
        let words = Self::count_words(text);
        let syllables = Self::count_syllables(text);

        let avg_words_per_sentence = if sentences > 0 {
            words as f32 / sentences as f32
        } else {
            0.0
        };

        let avg_syllables_per_word = if words > 0 {
            syllables as f32 / words as f32
        } else {
            0.0
        };

        // Flesch Reading Ease: 206.835 - 1.015(words/sentences) - 84.6(syllables/words)
        let flesch_score = 206.835
            - (1.015 * avg_words_per_sentence)
            - (84.6 * avg_syllables_per_word);

        // Flesch-Kincaid Grade Level: 0.39(words/sentences) + 11.8(syllables/words) - 15.59
        let grade_level =
            0.39 * avg_words_per_sentence + 11.8 * avg_syllables_per_word - 15.59;

        ReadabilityScore {
            avg_words_per_sentence,
            avg_syllables_per_word,
            flesch_score: flesch_score.max(0.0).min(100.0),
            grade_level: grade_level.max(0.0),
            word_count: words,
            sentence_count: sentences,
        }
    }

    /// Count sentences in text
    fn count_sentences(text: &str) -> usize {
        text.split(&['.', '!', '?'])
            .filter(|s| !s.trim().is_empty())
            .count()
            .max(1)
    }

    /// Count words in text
    fn count_words(text: &str) -> usize {
        text.unicode_words().count()
    }

    /// Count syllables in text (simplified algorithm)
    fn count_syllables(text: &str) -> usize {
        text.unicode_words()
            .map(|word| Self::count_syllables_in_word(word))
            .sum()
    }

    /// Count syllables in a single word
    fn count_syllables_in_word(word: &str) -> usize {
        let word = word.to_lowercase();
        let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];

        let mut count = 0;
        let mut prev_was_vowel = false;

        for ch in word.chars() {
            let is_vowel = vowels.contains(&ch);
            if is_vowel && !prev_was_vowel {
                count += 1;
            }
            prev_was_vowel = is_vowel;
        }

        // Adjust for silent e
        if word.ends_with('e') && count > 1 {
            count -= 1;
        }

        count.max(1)
    }

    /// Check if text length is appropriate for context
    pub fn check_text_length(text: &str, max_chars: usize) -> Result<()> {
        let length = text.chars().count();
        if length > max_chars {
            Err(A11yError::Generic(format!(
                "Text length {} exceeds maximum {}",
                length, max_chars
            )))
        } else {
            Ok(())
        }
    }

    /// Validate text properties
    pub fn validate_properties(props: &TextProperties) -> Result<()> {
        if !props.meets_size_requirements() {
            return Err(A11yError::TextSizeTooSmall(props.font_size));
        }

        if !props.spacing.meets_wcag(props.font_size) {
            return Err(A11yError::Generic(
                "Text spacing does not meet WCAG requirements".to_string(),
            ));
        }

        Ok(())
    }

    /// Calculate optimal line length (characters per line)
    pub fn optimal_line_length(font_size: f32) -> usize {
        // Optimal line length is 45-75 characters for readability
        // Adjust based on font size
        let base = 60;
        let adjustment = (font_size - 16.0) / 2.0;
        ((base as f32 + adjustment) as usize).max(45).min(75)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_spacing() {
        let spacing = TextSpacing::for_font_size(16.0);
        assert_eq!(spacing.line_height, 24.0);
        assert_eq!(spacing.paragraph_spacing, 32.0);
        assert!(spacing.meets_wcag(16.0));
    }

    #[test]
    fn test_large_text_detection() {
        let normal = TextProperties::new(16.0);
        assert!(!normal.is_large_text());

        let large = TextProperties {
            font_size: 18.0,
            ..TextProperties::new(18.0)
        };
        assert!(large.is_large_text());

        let bold_large = TextProperties {
            font_size: 14.0,
            font_weight: 700,
            ..TextProperties::new(14.0)
        };
        assert!(bold_large.is_large_text());
    }

    #[test]
    fn test_readability_analysis() {
        let text = "The quick brown fox jumps over the lazy dog. This is a simple sentence.";
        let score = TextAnalyzer::analyze_readability(text);

        assert!(score.word_count > 0);
        assert!(score.sentence_count > 0);
        assert!(score.flesch_score >= 0.0 && score.flesch_score <= 100.0);
    }

    #[test]
    fn test_syllable_counting() {
        assert_eq!(TextAnalyzer::count_syllables_in_word("cat"), 1);
        assert_eq!(TextAnalyzer::count_syllables_in_word("happy"), 2);
        assert_eq!(TextAnalyzer::count_syllables_in_word("beautiful"), 3);
    }

    #[test]
    fn test_optimal_line_length() {
        let length = TextAnalyzer::optimal_line_length(16.0);
        assert!(length >= 45 && length <= 75);
    }
}
