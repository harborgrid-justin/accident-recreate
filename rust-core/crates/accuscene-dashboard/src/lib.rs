//! AccuScene Enterprise Dashboard System v0.2.5
//!
//! A comprehensive mobile-responsive enterprise dashboard system with:
//! - Responsive layout engine with breakpoint support
//! - Widget system (metrics, charts, tables)
//! - State management with persistence
//! - Type-safe configuration
//! - Production-ready error handling
//!
//! # Example
//!
//! ```rust
//! use accuscene_dashboard::{
//!     config::{DashboardConfig, Breakpoint},
//!     state::{DashboardState, DashboardStateManager},
//!     widgets::{
//!         metrics::{MetricsWidget, MetricsConfig, MetricValue, MetricFormat},
//!         WidgetConfig, WidgetMetadata, WidgetType,
//!         DisplayOptions, InteractionOptions,
//!     },
//!     layout::WidgetLayout,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create dashboard configuration
//!     let config = DashboardConfig::new("My Dashboard".to_string());
//!
//!     // Create state manager
//!     let manager = DashboardStateManager::new();
//!     let dashboard_id = manager.create_dashboard(config).await?;
//!
//!     // Add a metrics widget
//!     let metadata = WidgetMetadata::new(
//!         "widget-1".to_string(),
//!         WidgetType::Metrics,
//!         "Key Metrics".to_string(),
//!     );
//!
//!     let widget_config = WidgetConfig {
//!         metadata,
//!         data_source: None,
//!         config: serde_json::json!({}),
//!         display: DisplayOptions::default(),
//!         interaction: InteractionOptions::default(),
//!     };
//!
//!     let layout = WidgetLayout::new("widget-1".to_string(), 0, 0, 6, 3);
//!
//!     manager.update_dashboard(&dashboard_id, |state| {
//!         state.add_widget(widget_config, layout)
//!     }).await?;
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod error;
pub mod layout;
pub mod persistence;
pub mod state;
pub mod widgets;

// Re-export commonly used types
pub use config::{Breakpoint, DashboardConfig, GridConfig, ThemeConfig};
pub use error::{DashboardError, DashboardResult};
pub use layout::{ResponsiveLayout, WidgetLayout};
pub use persistence::{FileStorage, InMemoryStorage, PersistenceStorage};
pub use state::{DashboardState, DashboardStateManager, WidgetState};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Dashboard builder for fluent API
pub struct DashboardBuilder {
    config: DashboardConfig,
}

impl DashboardBuilder {
    /// Create a new dashboard builder
    pub fn new(name: String) -> Self {
        Self {
            config: DashboardConfig::new(name),
        }
    }

    /// Set description
    pub fn description(mut self, description: String) -> Self {
        self.config.description = Some(description);
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: ThemeConfig) -> Self {
        self.config.theme = theme;
        self
    }

    /// Enable dark mode
    pub fn dark_mode(mut self) -> Self {
        self.config.theme.dark_mode = true;
        self
    }

    /// Set refresh interval
    pub fn refresh_interval(mut self, seconds: u32) -> Self {
        self.config.refresh.interval_seconds = seconds;
        self
    }

    /// Enable/disable animations
    pub fn animations(mut self, enabled: bool) -> Self {
        self.config.animations_enabled = enabled;
        self
    }

    /// Enable/disable drag and drop
    pub fn drag_drop(mut self, enabled: bool) -> Self {
        self.config.drag_drop_enabled = enabled;
        self
    }

    /// Set maximum widgets
    pub fn max_widgets(mut self, max: u32) -> Self {
        self.config.max_widgets = max;
        self
    }

    /// Build the dashboard configuration
    pub fn build(self) -> DashboardResult<DashboardConfig> {
        self.config.validate_config()?;
        Ok(self.config)
    }

    /// Build and create dashboard state
    pub fn build_state(self) -> DashboardResult<DashboardState> {
        let config = self.build()?;
        Ok(DashboardState::new(config))
    }
}

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::config::{Breakpoint, DashboardConfig, GridConfig, ThemeConfig};
    pub use crate::error::{DashboardError, DashboardResult};
    pub use crate::layout::{BreakpointLayout, ResponsiveLayout, WidgetLayout};
    pub use crate::persistence::{FileStorage, InMemoryStorage, PersistenceStorage};
    pub use crate::state::{DashboardState, DashboardStateManager, WidgetState};
    pub use crate::widgets::{
        charts::{ChartConfig, ChartType, ChartWidget, DataSeries},
        metrics::{MetricFormat, MetricValue, MetricsConfig, MetricsWidget},
        tables::{ColumnDef, ColumnType, TableConfig, TableWidget},
        DisplayOptions, InteractionOptions, RefreshStrategy, Widget, WidgetConfig, WidgetData,
        WidgetMetadata, WidgetType,
    };
    pub use crate::DashboardBuilder;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(VERSION, "0.2.5");
    }

    #[test]
    fn test_dashboard_builder() {
        let config = DashboardBuilder::new("Test Dashboard".to_string())
            .description("Test description".to_string())
            .dark_mode()
            .refresh_interval(60)
            .build()
            .unwrap();

        assert_eq!(config.name, "Test Dashboard");
        assert_eq!(config.description, Some("Test description".to_string()));
        assert!(config.theme.dark_mode);
        assert_eq!(config.refresh.interval_seconds, 60);
    }

    #[test]
    fn test_dashboard_builder_validation() {
        let mut config = DashboardConfig::new("Test".to_string());
        config.theme.primary_color = "invalid".to_string();

        let builder = DashboardBuilder { config };
        assert!(builder.build().is_err());
    }
}
