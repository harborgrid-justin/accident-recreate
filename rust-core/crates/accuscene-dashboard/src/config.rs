//! Dashboard configuration module
//!
//! Provides comprehensive configuration management for dashboard layouts,
//! themes, and behavior across different breakpoints

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use crate::error::{DashboardError, DashboardResult};

/// Responsive breakpoint definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Breakpoint {
    /// Mobile portrait (< 576px)
    Mobile,
    /// Mobile landscape / Small tablet (576px - 768px)
    MobileLandscape,
    /// Tablet (768px - 992px)
    Tablet,
    /// Desktop (992px - 1200px)
    Desktop,
    /// Large desktop (1200px - 1600px)
    DesktopLarge,
    /// Extra large desktop (> 1600px)
    DesktopXL,
}

impl Breakpoint {
    /// Get the minimum width for this breakpoint in pixels
    pub fn min_width(&self) -> u32 {
        match self {
            Breakpoint::Mobile => 0,
            Breakpoint::MobileLandscape => 576,
            Breakpoint::Tablet => 768,
            Breakpoint::Desktop => 992,
            Breakpoint::DesktopLarge => 1200,
            Breakpoint::DesktopXL => 1600,
        }
    }

    /// Get the maximum width for this breakpoint in pixels (None for unbounded)
    pub fn max_width(&self) -> Option<u32> {
        match self {
            Breakpoint::Mobile => Some(575),
            Breakpoint::MobileLandscape => Some(767),
            Breakpoint::Tablet => Some(991),
            Breakpoint::Desktop => Some(1199),
            Breakpoint::DesktopLarge => Some(1599),
            Breakpoint::DesktopXL => None,
        }
    }

    /// Get breakpoint from width in pixels
    pub fn from_width(width: u32) -> Self {
        if width < 576 {
            Breakpoint::Mobile
        } else if width < 768 {
            Breakpoint::MobileLandscape
        } else if width < 992 {
            Breakpoint::Tablet
        } else if width < 1200 {
            Breakpoint::Desktop
        } else if width < 1600 {
            Breakpoint::DesktopLarge
        } else {
            Breakpoint::DesktopXL
        }
    }

    /// Get recommended column count for this breakpoint
    pub fn default_columns(&self) -> u32 {
        match self {
            Breakpoint::Mobile => 4,
            Breakpoint::MobileLandscape => 6,
            Breakpoint::Tablet => 8,
            Breakpoint::Desktop => 12,
            Breakpoint::DesktopLarge => 12,
            Breakpoint::DesktopXL => 16,
        }
    }
}

/// Dashboard theme configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ThemeConfig {
    /// Primary color (hex)
    #[validate(length(equal = 7))]
    pub primary_color: String,

    /// Secondary color (hex)
    #[validate(length(equal = 7))]
    pub secondary_color: String,

    /// Background color (hex)
    #[validate(length(equal = 7))]
    pub background_color: String,

    /// Surface color (hex)
    #[validate(length(equal = 7))]
    pub surface_color: String,

    /// Text color (hex)
    #[validate(length(equal = 7))]
    pub text_color: String,

    /// Dark mode enabled
    pub dark_mode: bool,

    /// Border radius in pixels
    #[validate(range(min = 0, max = 50))]
    pub border_radius: u32,

    /// Spacing unit in pixels
    #[validate(range(min = 2, max = 32))]
    pub spacing_unit: u32,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            primary_color: "#1976d2".to_string(),
            secondary_color: "#dc004e".to_string(),
            background_color: "#f5f5f5".to_string(),
            surface_color: "#ffffff".to_string(),
            text_color: "#212121".to_string(),
            dark_mode: false,
            border_radius: 8,
            spacing_unit: 8,
        }
    }
}

/// Grid configuration for a specific breakpoint
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GridConfig {
    /// Number of columns
    #[validate(range(min = 1, max = 24))]
    pub columns: u32,

    /// Row height in pixels
    #[validate(range(min = 10, max = 500))]
    pub row_height: u32,

    /// Horizontal gap between widgets in pixels
    #[validate(range(min = 0, max = 100))]
    pub horizontal_gap: u32,

    /// Vertical gap between widgets in pixels
    #[validate(range(min = 0, max = 100))]
    pub vertical_gap: u32,

    /// Container padding in pixels
    #[validate(range(min = 0, max = 100))]
    pub container_padding: u32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            columns: 12,
            row_height: 80,
            horizontal_gap: 16,
            vertical_gap: 16,
            container_padding: 24,
        }
    }
}

impl GridConfig {
    /// Create a grid config optimized for a specific breakpoint
    pub fn for_breakpoint(breakpoint: Breakpoint) -> Self {
        match breakpoint {
            Breakpoint::Mobile => Self {
                columns: 4,
                row_height: 60,
                horizontal_gap: 8,
                vertical_gap: 8,
                container_padding: 12,
            },
            Breakpoint::MobileLandscape => Self {
                columns: 6,
                row_height: 70,
                horizontal_gap: 12,
                vertical_gap: 12,
                container_padding: 16,
            },
            Breakpoint::Tablet => Self {
                columns: 8,
                row_height: 80,
                horizontal_gap: 16,
                vertical_gap: 16,
                container_padding: 20,
            },
            Breakpoint::Desktop | Breakpoint::DesktopLarge => Self::default(),
            Breakpoint::DesktopXL => Self {
                columns: 16,
                row_height: 90,
                horizontal_gap: 20,
                vertical_gap: 20,
                container_padding: 32,
            },
        }
    }
}

/// Refresh configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RefreshConfig {
    /// Auto-refresh enabled
    pub enabled: bool,

    /// Refresh interval in seconds
    #[validate(range(min = 1, max = 3600))]
    pub interval_seconds: u32,

    /// Refresh on visibility change
    pub refresh_on_focus: bool,

    /// Stale data threshold in seconds
    #[validate(range(min = 1, max = 7200))]
    pub stale_threshold_seconds: u32,
}

impl Default for RefreshConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 30,
            refresh_on_focus: true,
            stale_threshold_seconds: 300,
        }
    }
}

/// Main dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DashboardConfig {
    /// Dashboard unique identifier
    pub id: String,

    /// Dashboard name
    #[validate(length(min = 1, max = 200))]
    pub name: String,

    /// Dashboard description
    #[validate(length(max = 1000))]
    pub description: Option<String>,

    /// Theme configuration
    #[validate]
    pub theme: ThemeConfig,

    /// Grid configurations per breakpoint
    pub grid_configs: HashMap<Breakpoint, GridConfig>,

    /// Refresh configuration
    #[validate]
    pub refresh: RefreshConfig,

    /// Enable animations
    pub animations_enabled: bool,

    /// Enable drag and drop
    pub drag_drop_enabled: bool,

    /// Enable widget resize
    pub resize_enabled: bool,

    /// Maximum number of widgets
    #[validate(range(min = 1, max = 100))]
    pub max_widgets: u32,

    /// Enable persistence
    pub persistence_enabled: bool,

    /// Version for optimistic locking
    pub version: u64,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        let mut grid_configs = HashMap::new();
        grid_configs.insert(Breakpoint::Mobile, GridConfig::for_breakpoint(Breakpoint::Mobile));
        grid_configs.insert(Breakpoint::MobileLandscape, GridConfig::for_breakpoint(Breakpoint::MobileLandscape));
        grid_configs.insert(Breakpoint::Tablet, GridConfig::for_breakpoint(Breakpoint::Tablet));
        grid_configs.insert(Breakpoint::Desktop, GridConfig::for_breakpoint(Breakpoint::Desktop));
        grid_configs.insert(Breakpoint::DesktopLarge, GridConfig::for_breakpoint(Breakpoint::DesktopLarge));
        grid_configs.insert(Breakpoint::DesktopXL, GridConfig::for_breakpoint(Breakpoint::DesktopXL));

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Dashboard".to_string(),
            description: None,
            theme: ThemeConfig::default(),
            grid_configs,
            refresh: RefreshConfig::default(),
            animations_enabled: true,
            drag_drop_enabled: true,
            resize_enabled: true,
            max_widgets: 50,
            persistence_enabled: true,
            version: 0,
        }
    }
}

impl DashboardConfig {
    /// Create a new dashboard configuration
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate_config(&self) -> DashboardResult<()> {
        self.validate()
            .map_err(|e| DashboardError::validation(format!("Configuration validation failed: {}", e)))?;

        // Validate theme colors are valid hex
        for color in &[
            &self.theme.primary_color,
            &self.theme.secondary_color,
            &self.theme.background_color,
            &self.theme.surface_color,
            &self.theme.text_color,
        ] {
            if !color.starts_with('#') || color.len() != 7 {
                return Err(DashboardError::validation(format!("Invalid color format: {}", color)));
            }
        }

        // Validate grid configs
        for (breakpoint, config) in &self.grid_configs {
            config.validate()
                .map_err(|e| DashboardError::validation(
                    format!("Grid config validation failed for {:?}: {}", breakpoint, e)
                ))?;
        }

        Ok(())
    }

    /// Get grid config for a specific breakpoint
    pub fn grid_for_breakpoint(&self, breakpoint: Breakpoint) -> GridConfig {
        self.grid_configs
            .get(&breakpoint)
            .cloned()
            .unwrap_or_else(|| GridConfig::for_breakpoint(breakpoint))
    }

    /// Get grid config for a specific width
    pub fn grid_for_width(&self, width: u32) -> GridConfig {
        let breakpoint = Breakpoint::from_width(width);
        self.grid_for_breakpoint(breakpoint)
    }

    /// Update theme
    pub fn with_theme(mut self, theme: ThemeConfig) -> Self {
        self.theme = theme;
        self
    }

    /// Enable dark mode
    pub fn with_dark_mode(mut self, enabled: bool) -> Self {
        self.theme.dark_mode = enabled;
        self
    }

    /// Set refresh interval
    pub fn with_refresh_interval(mut self, seconds: u32) -> Self {
        self.refresh.interval_seconds = seconds;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_from_width() {
        assert_eq!(Breakpoint::from_width(400), Breakpoint::Mobile);
        assert_eq!(Breakpoint::from_width(600), Breakpoint::MobileLandscape);
        assert_eq!(Breakpoint::from_width(800), Breakpoint::Tablet);
        assert_eq!(Breakpoint::from_width(1100), Breakpoint::Desktop);
        assert_eq!(Breakpoint::from_width(1400), Breakpoint::DesktopLarge);
        assert_eq!(Breakpoint::from_width(1800), Breakpoint::DesktopXL);
    }

    #[test]
    fn test_default_config() {
        let config = DashboardConfig::default();
        assert!(config.validate_config().is_ok());
        assert_eq!(config.grid_configs.len(), 6);
    }

    #[test]
    fn test_grid_for_width() {
        let config = DashboardConfig::default();
        let grid = config.grid_for_width(500);
        assert_eq!(grid.columns, 4); // Mobile
    }

    #[test]
    fn test_theme_validation() {
        let mut config = DashboardConfig::default();
        config.theme.primary_color = "invalid".to_string();
        assert!(config.validate_config().is_err());
    }
}
