//! Accessibility configuration and settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WCAG conformance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WcagLevel {
    /// Level A - Minimum conformance
    A,
    /// Level AA - Mid-range conformance (recommended)
    AA,
    /// Level AAA - Highest conformance
    AAA,
}

impl Default for WcagLevel {
    fn default() -> Self {
        Self::AA
    }
}

/// Color scheme preferences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    Light,
    Dark,
    HighContrast,
    Auto,
}

/// Motion preferences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionPreference {
    NoPreference,
    Reduce,
    None,
}

/// Text size preferences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl TextSize {
    /// Get the scale factor for this text size
    pub fn scale_factor(&self) -> f32 {
        match self {
            Self::Small => 0.875,
            Self::Medium => 1.0,
            Self::Large => 1.25,
            Self::ExtraLarge => 1.5,
        }
    }
}

/// Comprehensive accessibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yConfig {
    /// WCAG conformance level
    wcag_level: WcagLevel,

    /// Enable screen reader support
    screen_reader_enabled: bool,

    /// Enable keyboard navigation
    keyboard_nav_enabled: bool,

    /// Enable focus indicators
    focus_indicators_enabled: bool,

    /// Color scheme preference
    color_scheme: ColorScheme,

    /// Motion preference
    motion_preference: MotionPreference,

    /// Text size preference
    text_size: TextSize,

    /// Enable reduced transparency
    reduce_transparency: bool,

    /// Enable captions
    captions_enabled: bool,

    /// Enable audio descriptions
    audio_descriptions_enabled: bool,

    /// Timeout duration in seconds (0 = no timeout)
    timeout_duration: u32,

    /// Custom ARIA label overrides
    aria_labels: HashMap<String, String>,

    /// Language code (ISO 639-1)
    language: String,

    /// Text direction
    text_direction: TextDirection,
}

/// Text direction for internationalization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
    Auto,
}

impl Default for A11yConfig {
    fn default() -> Self {
        Self {
            wcag_level: WcagLevel::AA,
            screen_reader_enabled: true,
            keyboard_nav_enabled: true,
            focus_indicators_enabled: true,
            color_scheme: ColorScheme::Auto,
            motion_preference: MotionPreference::NoPreference,
            text_size: TextSize::Medium,
            reduce_transparency: false,
            captions_enabled: false,
            audio_descriptions_enabled: false,
            timeout_duration: 0,
            aria_labels: HashMap::new(),
            language: "en".to_string(),
            text_direction: TextDirection::Auto,
        }
    }
}

impl A11yConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set WCAG level
    pub fn with_wcag_level(mut self, level: WcagLevel) -> Self {
        self.wcag_level = level;
        self
    }

    /// Enable or disable screen reader support
    pub fn with_screen_reader_support(mut self, enabled: bool) -> Self {
        self.screen_reader_enabled = enabled;
        self
    }

    /// Enable or disable keyboard navigation
    pub fn with_keyboard_nav(mut self, enabled: bool) -> Self {
        self.keyboard_nav_enabled = enabled;
        self
    }

    /// Set color scheme
    pub fn with_color_scheme(mut self, scheme: ColorScheme) -> Self {
        self.color_scheme = scheme;
        self
    }

    /// Set motion preference
    pub fn with_motion_preference(mut self, preference: MotionPreference) -> Self {
        self.motion_preference = preference;
        self
    }

    /// Set text size
    pub fn with_text_size(mut self, size: TextSize) -> Self {
        self.text_size = size;
        self
    }

    /// Set language
    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = lang.into();
        self
    }

    /// Get WCAG level
    pub fn wcag_level(&self) -> WcagLevel {
        self.wcag_level
    }

    /// Check if screen reader support is enabled
    pub fn is_screen_reader_enabled(&self) -> bool {
        self.screen_reader_enabled
    }

    /// Check if keyboard navigation is enabled
    pub fn is_keyboard_nav_enabled(&self) -> bool {
        self.keyboard_nav_enabled
    }

    /// Check if focus indicators are enabled
    pub fn are_focus_indicators_enabled(&self) -> bool {
        self.focus_indicators_enabled
    }

    /// Get color scheme
    pub fn color_scheme(&self) -> ColorScheme {
        self.color_scheme
    }

    /// Get motion preference
    pub fn motion_preference(&self) -> MotionPreference {
        self.motion_preference
    }

    /// Get text size
    pub fn text_size(&self) -> TextSize {
        self.text_size
    }

    /// Get minimum contrast ratio for current WCAG level
    pub fn min_contrast_ratio(&self) -> f64 {
        match self.wcag_level {
            WcagLevel::A => 3.0,
            WcagLevel::AA => 4.5,
            WcagLevel::AAA => 7.0,
        }
    }

    /// Get minimum contrast ratio for large text
    pub fn min_contrast_ratio_large_text(&self) -> f64 {
        match self.wcag_level {
            WcagLevel::A => 3.0,
            WcagLevel::AA => 3.0,
            WcagLevel::AAA => 4.5,
        }
    }

    /// Add custom ARIA label
    pub fn add_aria_label(&mut self, key: String, label: String) {
        self.aria_labels.insert(key, label);
    }

    /// Get custom ARIA label
    pub fn get_aria_label(&self, key: &str) -> Option<&str> {
        self.aria_labels.get(key).map(|s| s.as_str())
    }

    /// Get language code
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Get text direction
    pub fn text_direction(&self) -> TextDirection {
        self.text_direction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = A11yConfig::default();
        assert_eq!(config.wcag_level(), WcagLevel::AA);
        assert!(config.is_screen_reader_enabled());
        assert!(config.is_keyboard_nav_enabled());
    }

    #[test]
    fn test_wcag_contrast_ratios() {
        let aa_config = A11yConfig::default().with_wcag_level(WcagLevel::AA);
        assert_eq!(aa_config.min_contrast_ratio(), 4.5);
        assert_eq!(aa_config.min_contrast_ratio_large_text(), 3.0);

        let aaa_config = A11yConfig::default().with_wcag_level(WcagLevel::AAA);
        assert_eq!(aaa_config.min_contrast_ratio(), 7.0);
        assert_eq!(aaa_config.min_contrast_ratio_large_text(), 4.5);
    }

    #[test]
    fn test_text_size_scale() {
        assert_eq!(TextSize::Small.scale_factor(), 0.875);
        assert_eq!(TextSize::Medium.scale_factor(), 1.0);
        assert_eq!(TextSize::Large.scale_factor(), 1.25);
        assert_eq!(TextSize::ExtraLarge.scale_factor(), 1.5);
    }
}
