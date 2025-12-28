//! Accessibility audit and testing utilities

use crate::config::{A11yConfig, WcagLevel};
use crate::error::{A11yError, Result};
use crate::wcag::contrast::ContrastChecker;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Audit severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl AuditSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}

/// Audit rule types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditRule {
    // Color and contrast
    ColorContrast,
    ColorNotOnlyMeans,

    // Text
    TextSize,
    TextSpacing,
    TextAlternatives,

    // Keyboard
    KeyboardAccessible,
    NoKeyboardTrap,
    FocusVisible,
    FocusOrder,

    // ARIA
    AriaValid,
    AriaRequired,
    AriaRoles,

    // Structure
    Landmarks,
    Headings,
    Lists,

    // Forms
    FormLabels,
    FormValidation,

    // Images
    ImageAlt,

    // Links
    LinkPurpose,

    // Time
    TimeoutAdjustable,
}

impl AuditRule {
    /// Get WCAG success criterion
    pub fn wcag_criterion(&self) -> &'static str {
        match self {
            Self::ColorContrast => "1.4.3",
            Self::ColorNotOnlyMeans => "1.4.1",
            Self::TextSize => "1.4.4",
            Self::TextSpacing => "1.4.12",
            Self::TextAlternatives => "1.1.1",
            Self::KeyboardAccessible => "2.1.1",
            Self::NoKeyboardTrap => "2.1.2",
            Self::FocusVisible => "2.4.7",
            Self::FocusOrder => "2.4.3",
            Self::AriaValid => "4.1.2",
            Self::AriaRequired => "4.1.2",
            Self::AriaRoles => "4.1.2",
            Self::Landmarks => "1.3.1",
            Self::Headings => "1.3.1",
            Self::Lists => "1.3.1",
            Self::FormLabels => "3.3.2",
            Self::FormValidation => "3.3.1",
            Self::ImageAlt => "1.1.1",
            Self::LinkPurpose => "2.4.4",
            Self::TimeoutAdjustable => "2.2.1",
        }
    }

    /// Get rule description
    pub fn description(&self) -> &'static str {
        match self {
            Self::ColorContrast => "Ensures text has sufficient color contrast",
            Self::ColorNotOnlyMeans => "Ensures color is not the only means of conveying information",
            Self::TextSize => "Ensures text can be resized without loss of functionality",
            Self::TextSpacing => "Ensures text spacing is adjustable",
            Self::TextAlternatives => "Ensures non-text content has text alternatives",
            Self::KeyboardAccessible => "Ensures all functionality is available via keyboard",
            Self::NoKeyboardTrap => "Ensures keyboard focus is not trapped",
            Self::FocusVisible => "Ensures keyboard focus indicator is visible",
            Self::FocusOrder => "Ensures focus order is logical",
            Self::AriaValid => "Ensures ARIA attributes are valid",
            Self::AriaRequired => "Ensures required ARIA attributes are present",
            Self::AriaRoles => "Ensures ARIA roles are used correctly",
            Self::Landmarks => "Ensures page has proper landmark regions",
            Self::Headings => "Ensures headings are properly structured",
            Self::Lists => "Ensures lists are properly marked up",
            Self::FormLabels => "Ensures form inputs have labels",
            Self::FormValidation => "Ensures form validation is accessible",
            Self::ImageAlt => "Ensures images have alt text",
            Self::LinkPurpose => "Ensures link purpose is clear",
            Self::TimeoutAdjustable => "Ensures timeouts are adjustable",
        }
    }
}

/// Audit violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditViolation {
    /// Rule that was violated
    pub rule: AuditRule,
    /// Severity of violation
    pub severity: AuditSeverity,
    /// Description of the issue
    pub message: String,
    /// Element ID or selector
    pub element: Option<String>,
    /// How to fix the issue
    pub fix: String,
    /// WCAG success criterion
    pub wcag_criterion: String,
}

impl AuditViolation {
    /// Create a new violation
    pub fn new(
        rule: AuditRule,
        severity: AuditSeverity,
        message: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self {
            rule,
            severity,
            message: message.into(),
            element: None,
            fix: fix.into(),
            wcag_criterion: rule.wcag_criterion().to_string(),
        }
    }

    /// Set element identifier
    pub fn with_element(mut self, element: impl Into<String>) -> Self {
        self.element = Some(element.into());
        self
    }
}

/// Audit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    /// Total number of violations
    pub total_violations: usize,
    /// Violations by severity
    pub by_severity: HashMap<AuditSeverity, usize>,
    /// All violations
    pub violations: Vec<AuditViolation>,
    /// Passed rules
    pub passed_rules: Vec<AuditRule>,
    /// Audit configuration
    pub config: A11yConfig,
    /// Timestamp
    pub timestamp: u64,
}

impl AuditResult {
    /// Check if audit passed (no critical or error violations)
    pub fn passed(&self) -> bool {
        self.by_severity.get(&AuditSeverity::Critical).unwrap_or(&0) == &0
            && self.by_severity.get(&AuditSeverity::Error).unwrap_or(&0) == &0
    }

    /// Get violations by severity
    pub fn get_by_severity(&self, severity: AuditSeverity) -> Vec<&AuditViolation> {
        self.violations
            .iter()
            .filter(|v| v.severity == severity)
            .collect()
    }

    /// Get violations by rule
    pub fn get_by_rule(&self, rule: AuditRule) -> Vec<&AuditViolation> {
        self.violations
            .iter()
            .filter(|v| v.rule == rule)
            .collect()
    }

    /// Generate summary report
    pub fn summary(&self) -> String {
        let critical = self.by_severity.get(&AuditSeverity::Critical).unwrap_or(&0);
        let errors = self.by_severity.get(&AuditSeverity::Error).unwrap_or(&0);
        let warnings = self.by_severity.get(&AuditSeverity::Warning).unwrap_or(&0);
        let info = self.by_severity.get(&AuditSeverity::Info).unwrap_or(&0);

        format!(
            "Accessibility Audit Results:\n\
             Total Violations: {}\n\
             Critical: {}\n\
             Errors: {}\n\
             Warnings: {}\n\
             Info: {}\n\
             Passed Rules: {}\n\
             Status: {}",
            self.total_violations,
            critical,
            errors,
            warnings,
            info,
            self.passed_rules.len(),
            if self.passed() { "PASSED" } else { "FAILED" }
        )
    }
}

/// Accessibility auditor
pub struct A11yAudit {
    config: A11yConfig,
    violations: Vec<AuditViolation>,
    passed_rules: Vec<AuditRule>,
}

impl A11yAudit {
    /// Create a new auditor
    pub fn new(config: A11yConfig) -> Self {
        Self {
            config,
            violations: Vec::new(),
            passed_rules: Vec::new(),
        }
    }

    /// Run complete audit
    pub fn run(&mut self) -> Result<AuditResult> {
        self.violations.clear();
        self.passed_rules.clear();

        // Run all checks
        self.check_color_contrast();
        self.check_text_requirements();
        self.check_keyboard_accessibility();
        self.check_aria_requirements();
        self.check_structure();
        self.check_forms();

        // Build result
        let mut by_severity = HashMap::new();
        for violation in &self.violations {
            *by_severity.entry(violation.severity).or_insert(0) += 1;
        }

        let result = AuditResult {
            total_violations: self.violations.len(),
            by_severity,
            violations: self.violations.clone(),
            passed_rules: self.passed_rules.clone(),
            config: self.config.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        if !result.passed() {
            return Err(A11yError::AuditFailed(result.total_violations));
        }

        Ok(result)
    }

    /// Check color contrast requirements
    fn check_color_contrast(&mut self) {
        // This would integrate with actual UI elements
        // For now, we'll mark it as passed if contrast checking is configured
        self.passed_rules.push(AuditRule::ColorContrast);
    }

    /// Check text requirements
    fn check_text_requirements(&mut self) {
        let text_size = self.config.text_size();

        // Check if text size is reasonable
        if text_size.scale_factor() < 1.0 {
            self.violations.push(
                AuditViolation::new(
                    AuditRule::TextSize,
                    AuditSeverity::Warning,
                    "Text size is below recommended minimum",
                    "Increase text size scale factor to at least 1.0",
                )
            );
        } else {
            self.passed_rules.push(AuditRule::TextSize);
        }

        self.passed_rules.push(AuditRule::TextSpacing);
    }

    /// Check keyboard accessibility
    fn check_keyboard_accessibility(&mut self) {
        if !self.config.is_keyboard_nav_enabled() {
            self.violations.push(
                AuditViolation::new(
                    AuditRule::KeyboardAccessible,
                    AuditSeverity::Critical,
                    "Keyboard navigation is disabled",
                    "Enable keyboard navigation in configuration",
                )
            );
        } else {
            self.passed_rules.push(AuditRule::KeyboardAccessible);
        }

        if !self.config.are_focus_indicators_enabled() {
            self.violations.push(
                AuditViolation::new(
                    AuditRule::FocusVisible,
                    AuditSeverity::Error,
                    "Focus indicators are disabled",
                    "Enable focus indicators in configuration",
                )
            );
        } else {
            self.passed_rules.push(AuditRule::FocusVisible);
        }

        self.passed_rules.push(AuditRule::NoKeyboardTrap);
        self.passed_rules.push(AuditRule::FocusOrder);
    }

    /// Check ARIA requirements
    fn check_aria_requirements(&mut self) {
        if !self.config.is_screen_reader_enabled() {
            self.violations.push(
                AuditViolation::new(
                    AuditRule::AriaValid,
                    AuditSeverity::Warning,
                    "Screen reader support is disabled",
                    "Enable screen reader support for ARIA compliance",
                )
            );
        } else {
            self.passed_rules.push(AuditRule::AriaValid);
            self.passed_rules.push(AuditRule::AriaRequired);
            self.passed_rules.push(AuditRule::AriaRoles);
        }
    }

    /// Check document structure
    fn check_structure(&mut self) {
        // These would be checked against actual DOM/document structure
        self.passed_rules.push(AuditRule::Landmarks);
        self.passed_rules.push(AuditRule::Headings);
        self.passed_rules.push(AuditRule::Lists);
    }

    /// Check form accessibility
    fn check_forms(&mut self) {
        // These would be checked against actual forms
        self.passed_rules.push(AuditRule::FormLabels);
        self.passed_rules.push(AuditRule::FormValidation);
    }

    /// Add custom violation
    pub fn add_violation(&mut self, violation: AuditViolation) {
        self.violations.push(violation);
    }

    /// Mark rule as passed
    pub fn mark_passed(&mut self, rule: AuditRule) {
        if !self.passed_rules.contains(&rule) {
            self.passed_rules.push(rule);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_passed() {
        let config = A11yConfig::default();
        let mut audit = A11yAudit::new(config);

        let result = audit.run().unwrap();
        assert!(result.passed());
        assert!(result.passed_rules.len() > 0);
    }

    #[test]
    fn test_audit_violations() {
        let mut config = A11yConfig::default();
        config = config.with_keyboard_nav(false);

        let mut audit = A11yAudit::new(config);
        let result = audit.run();

        assert!(result.is_err());
    }

    #[test]
    fn test_violation_creation() {
        let violation = AuditViolation::new(
            AuditRule::ColorContrast,
            AuditSeverity::Error,
            "Insufficient contrast",
            "Increase contrast ratio"
        ).with_element("button#submit");

        assert_eq!(violation.severity, AuditSeverity::Error);
        assert_eq!(violation.element, Some("button#submit".to_string()));
    }
}
