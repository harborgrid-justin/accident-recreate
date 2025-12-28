//! Log level filtering

use std::str::FromStr;

/// Log level filter
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LevelFilter {
    /// Trace level
    Trace,
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warn level
    Warn,
    /// Error level
    Error,
    /// Off (no logging)
    Off,
}

impl LevelFilter {
    /// Check if a level should be logged
    pub fn should_log(&self, level: &LevelFilter) -> bool {
        level >= self
    }

    /// Convert to tracing level
    pub fn to_tracing_level(&self) -> Option<tracing::Level> {
        match self {
            Self::Trace => Some(tracing::Level::TRACE),
            Self::Debug => Some(tracing::Level::DEBUG),
            Self::Info => Some(tracing::Level::INFO),
            Self::Warn => Some(tracing::Level::WARN),
            Self::Error => Some(tracing::Level::ERROR),
            Self::Off => None,
        }
    }

    /// Get all log levels
    pub fn all() -> &'static [LevelFilter] {
        &[
            LevelFilter::Trace,
            LevelFilter::Debug,
            LevelFilter::Info,
            LevelFilter::Warn,
            LevelFilter::Error,
            LevelFilter::Off,
        ]
    }
}

impl FromStr for LevelFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(Self::Trace),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" | "warning" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            "off" => Ok(Self::Off),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }
}

impl std::fmt::Display for LevelFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trace => write!(f, "TRACE"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
            Self::Off => write!(f, "OFF"),
        }
    }
}

/// Module-specific log filter
#[derive(Debug, Clone)]
pub struct ModuleFilter {
    module: String,
    level: LevelFilter,
}

impl ModuleFilter {
    /// Create a new module filter
    pub fn new(module: impl Into<String>, level: LevelFilter) -> Self {
        Self {
            module: module.into(),
            level,
        }
    }

    /// Check if a module and level should be logged
    pub fn should_log(&self, module: &str, level: &LevelFilter) -> bool {
        module.starts_with(&self.module) && self.level.should_log(level)
    }

    /// Get the module name
    pub fn module(&self) -> &str {
        &self.module
    }

    /// Get the log level
    pub fn level(&self) -> LevelFilter {
        self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_ordering() {
        assert!(LevelFilter::Error > LevelFilter::Warn);
        assert!(LevelFilter::Warn > LevelFilter::Info);
        assert!(LevelFilter::Info > LevelFilter::Debug);
        assert!(LevelFilter::Debug > LevelFilter::Trace);
    }

    #[test]
    fn test_should_log() {
        let filter = LevelFilter::Info;
        assert!(filter.should_log(&LevelFilter::Error));
        assert!(filter.should_log(&LevelFilter::Warn));
        assert!(filter.should_log(&LevelFilter::Info));
        assert!(!filter.should_log(&LevelFilter::Debug));
        assert!(!filter.should_log(&LevelFilter::Trace));
    }

    #[test]
    fn test_from_str() {
        assert_eq!(LevelFilter::from_str("info").unwrap(), LevelFilter::Info);
        assert_eq!(LevelFilter::from_str("WARN").unwrap(), LevelFilter::Warn);
        assert!(LevelFilter::from_str("invalid").is_err());
    }
}
