//! WCAG 2.1 compliance checking modules

pub mod contrast;
pub mod navigation;
pub mod text;

pub use contrast::{ContrastChecker, ContrastRatio, ContrastResult};
pub use navigation::{KeyboardNavigator, NavigationPattern};
pub use text::{ReadabilityScore, TextAnalyzer};
