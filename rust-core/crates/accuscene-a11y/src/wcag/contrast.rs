//! Color contrast checking for WCAG compliance
//!
//! Implements WCAG 2.1 Success Criterion 1.4.3 (Contrast Minimum)
//! and 1.4.6 (Contrast Enhanced)

use crate::error::{A11yError, Result};
use crate::config::WcagLevel;
use serde::{Deserialize, Serialize};

/// Color contrast ratio
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ContrastRatio(pub f64);

impl ContrastRatio {
    /// Create a new contrast ratio
    pub fn new(ratio: f64) -> Self {
        Self(ratio.max(1.0).min(21.0))
    }

    /// Get the ratio value
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if ratio meets WCAG AA for normal text
    pub fn meets_aa_normal(&self) -> bool {
        self.0 >= 4.5
    }

    /// Check if ratio meets WCAG AA for large text
    pub fn meets_aa_large(&self) -> bool {
        self.0 >= 3.0
    }

    /// Check if ratio meets WCAG AAA for normal text
    pub fn meets_aaa_normal(&self) -> bool {
        self.0 >= 7.0
    }

    /// Check if ratio meets WCAG AAA for large text
    pub fn meets_aaa_large(&self) -> bool {
        self.0 >= 4.5
    }

    /// Check if ratio meets specified WCAG level
    pub fn meets_level(&self, level: WcagLevel, is_large_text: bool) -> bool {
        match (level, is_large_text) {
            (WcagLevel::A, _) => self.0 >= 3.0,
            (WcagLevel::AA, false) => self.meets_aa_normal(),
            (WcagLevel::AA, true) => self.meets_aa_large(),
            (WcagLevel::AAA, false) => self.meets_aaa_normal(),
            (WcagLevel::AAA, true) => self.meets_aaa_large(),
        }
    }
}

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    /// Create a new RGB color
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse from hex string (e.g., "#FF0000" or "FF0000")
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 {
            return Err(A11yError::InvalidColor(format!(
                "Hex color must be 6 characters, got {}",
                hex.len()
            )));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| A11yError::InvalidColor("Invalid red component".to_string()))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| A11yError::InvalidColor("Invalid green component".to_string()))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| A11yError::InvalidColor("Invalid blue component".to_string()))?;

        Ok(Self { r, g, b })
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Calculate relative luminance (WCAG formula)
    pub fn relative_luminance(&self) -> f64 {
        fn linearize(component: u8) -> f64 {
            let c = component as f64 / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }

        let r = linearize(self.r);
        let g = linearize(self.g);
        let b = linearize(self.b);

        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
}

/// Contrast check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContrastResult {
    pub foreground: RgbColor,
    pub background: RgbColor,
    pub ratio: ContrastRatio,
    pub meets_aa_normal: bool,
    pub meets_aa_large: bool,
    pub meets_aaa_normal: bool,
    pub meets_aaa_large: bool,
}

impl ContrastResult {
    /// Check if result meets specified WCAG level
    pub fn meets_level(&self, level: WcagLevel, is_large_text: bool) -> bool {
        self.ratio.meets_level(level, is_large_text)
    }

    /// Get recommendation message
    pub fn recommendation(&self, level: WcagLevel, is_large_text: bool) -> Option<String> {
        if self.meets_level(level, is_large_text) {
            None
        } else {
            let required = match (level, is_large_text) {
                (WcagLevel::A, _) => 3.0,
                (WcagLevel::AA, false) => 4.5,
                (WcagLevel::AA, true) => 3.0,
                (WcagLevel::AAA, false) => 7.0,
                (WcagLevel::AAA, true) => 4.5,
            };
            Some(format!(
                "Contrast ratio {:.2} is below required {:.2} for {:?} {}",
                self.ratio.value(),
                required,
                level,
                if is_large_text { "(large text)" } else { "(normal text)" }
            ))
        }
    }
}

/// Color contrast checker
pub struct ContrastChecker {
    /// Cached luminance values for performance
    luminance_cache: std::collections::HashMap<String, f64>,
}

impl Default for ContrastChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl ContrastChecker {
    /// Create a new contrast checker
    pub fn new() -> Self {
        Self {
            luminance_cache: std::collections::HashMap::new(),
        }
    }

    /// Calculate contrast ratio between two colors
    pub fn calculate_ratio(&self, color1: &RgbColor, color2: &RgbColor) -> ContrastRatio {
        let l1 = color1.relative_luminance();
        let l2 = color2.relative_luminance();

        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };

        let ratio = (lighter + 0.05) / (darker + 0.05);
        ContrastRatio::new(ratio)
    }

    /// Check contrast between two hex colors
    pub fn check_contrast(&self, foreground: &str, background: &str) -> Result<ContrastResult> {
        let fg = RgbColor::from_hex(foreground)?;
        let bg = RgbColor::from_hex(background)?;

        let ratio = self.calculate_ratio(&fg, &bg);

        Ok(ContrastResult {
            foreground: fg,
            background: bg,
            ratio,
            meets_aa_normal: ratio.meets_aa_normal(),
            meets_aa_large: ratio.meets_aa_large(),
            meets_aaa_normal: ratio.meets_aaa_normal(),
            meets_aaa_large: ratio.meets_aaa_large(),
        })
    }

    /// Suggest foreground color to meet contrast requirements
    pub fn suggest_foreground(
        &self,
        background: &str,
        level: WcagLevel,
        is_large_text: bool,
    ) -> Result<RgbColor> {
        let bg = RgbColor::from_hex(background)?;
        let bg_luminance = bg.relative_luminance();

        let required_ratio = match (level, is_large_text) {
            (WcagLevel::A, _) => 3.0,
            (WcagLevel::AA, false) => 4.5,
            (WcagLevel::AA, true) => 3.0,
            (WcagLevel::AAA, false) => 7.0,
            (WcagLevel::AAA, true) => 4.5,
        };

        // Try black first
        let black = RgbColor::new(0, 0, 0);
        if self.calculate_ratio(&black, &bg).value() >= required_ratio {
            return Ok(black);
        }

        // Try white
        let white = RgbColor::new(255, 255, 255);
        if self.calculate_ratio(&white, &bg).value() >= required_ratio {
            return Ok(white);
        }

        // If neither works, return the one with better contrast
        Ok(if bg_luminance > 0.5 { black } else { white })
    }

    /// Batch check multiple color pairs
    pub fn check_multiple(&self, pairs: Vec<(String, String)>) -> Vec<Result<ContrastResult>> {
        pairs
            .into_iter()
            .map(|(fg, bg)| self.check_contrast(&fg, &bg))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_from_hex() {
        let color = RgbColor::from_hex("#FF0000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);

        let color = RgbColor::from_hex("00FF00").unwrap();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_contrast_black_white() {
        let checker = ContrastChecker::new();
        let result = checker.check_contrast("#000000", "#FFFFFF").unwrap();

        assert_eq!(result.ratio.value(), 21.0);
        assert!(result.meets_aa_normal);
        assert!(result.meets_aa_large);
        assert!(result.meets_aaa_normal);
        assert!(result.meets_aaa_large);
    }

    #[test]
    fn test_contrast_ratio_aa() {
        let checker = ContrastChecker::new();
        let result = checker.check_contrast("#767676", "#FFFFFF").unwrap();

        assert!(result.ratio.value() >= 4.5);
        assert!(result.meets_aa_normal);
    }

    #[test]
    fn test_relative_luminance() {
        let black = RgbColor::new(0, 0, 0);
        assert_eq!(black.relative_luminance(), 0.0);

        let white = RgbColor::new(255, 255, 255);
        assert_eq!(white.relative_luminance(), 1.0);
    }

    #[test]
    fn test_suggest_foreground() {
        let checker = ContrastChecker::new();

        let fg = checker.suggest_foreground("#FFFFFF", WcagLevel::AA, false).unwrap();
        assert_eq!(fg, RgbColor::new(0, 0, 0));

        let fg = checker.suggest_foreground("#000000", WcagLevel::AA, false).unwrap();
        assert_eq!(fg, RgbColor::new(255, 255, 255));
    }
}
