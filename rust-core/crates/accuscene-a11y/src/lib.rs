//! AccuScene Accessibility System
//!
//! Comprehensive accessibility library implementing WCAG 2.1 AA standards,
//! screen reader support, keyboard navigation, and inclusive design patterns.
//!
//! # Features
//!
//! - **WCAG Compliance**: Color contrast checking, text readability, keyboard navigation
//! - **Screen Reader Support**: ARIA attribute generation, live announcements
//! - **Focus Management**: Programmatic focus control, focus trapping, focus restoration
//! - **Accessibility Auditing**: Automated testing and compliance verification
//!
//! # Example
//!
//! ```rust
//! use accuscene_a11y::prelude::*;
//!
//! let config = A11yConfig::default()
//!     .with_wcag_level(WcagLevel::AA)
//!     .with_screen_reader_support(true);
//!
//! let checker = ContrastChecker::new();
//! let result = checker.check_contrast("#000000", "#FFFFFF");
//! assert!(result.is_ok());
//! ```

pub mod config;
pub mod error;
pub mod focus;
pub mod screen_reader;
pub mod testing;
pub mod wcag;

pub use config::{A11yConfig, WcagLevel};
pub use error::{A11yError, Result};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::config::{A11yConfig, WcagLevel};
    pub use crate::error::{A11yError, Result};
    pub use crate::focus::management::{FocusManager, FocusNode, FocusTrap};
    pub use crate::screen_reader::aria::{AriaBuilder, AriaRole, AriaState};
    pub use crate::screen_reader::announcements::{
        AnnouncementPriority, LiveRegion, ScreenReaderAnnouncer,
    };
    pub use crate::testing::audit::{A11yAudit, AuditResult, AuditSeverity};
    pub use crate::wcag::contrast::{ContrastChecker, ContrastRatio, ContrastResult};
    pub use crate::wcag::navigation::{KeyboardNavigator, NavigationPattern};
    pub use crate::wcag::text::{ReadabilityScore, TextAnalyzer};
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_library_integration() {
        let config = A11yConfig::default();
        assert_eq!(config.wcag_level(), WcagLevel::AA);
    }
}
