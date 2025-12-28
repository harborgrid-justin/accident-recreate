//! Dashboard state management
//!
//! Provides centralized state management for dashboard instances

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::{Breakpoint, DashboardConfig};
use crate::error::{DashboardError, DashboardResult};
use crate::layout::{ResponsiveLayout, WidgetLayout};
use crate::widgets::{WidgetConfig, WidgetData};

/// Dashboard state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardState {
    /// Dashboard configuration
    pub config: DashboardConfig,

    /// Responsive layout
    pub layout: ResponsiveLayout,

    /// Widget states
    pub widgets: HashMap<String, WidgetState>,

    /// User preferences
    pub preferences: UserPreferences,

    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,

    /// State version for optimistic locking
    pub version: u64,
}

/// Individual widget state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetState {
    /// Widget configuration
    pub config: WidgetConfig,

    /// Widget data
    pub data: Option<WidgetData>,

    /// Loading state
    pub is_loading: bool,

    /// Error state
    pub error: Option<String>,

    /// Last refresh timestamp
    pub last_refresh: Option<chrono::DateTime<chrono::Utc>>,

    /// Widget-specific state (collapsed, expanded, etc.)
    pub ui_state: HashMap<String, serde_json::Value>,
}

impl WidgetState {
    /// Create a new widget state
    pub fn new(config: WidgetConfig) -> Self {
        Self {
            config,
            data: None,
            is_loading: false,
            error: None,
            last_refresh: None,
            ui_state: HashMap::new(),
        }
    }

    /// Check if data is stale
    pub fn is_stale(&self, ttl_seconds: u32) -> bool {
        if let Some(last_refresh) = self.last_refresh {
            let now = chrono::Utc::now();
            let age = now.signed_duration_since(last_refresh);
            age.num_seconds() as u32 > ttl_seconds
        } else {
            true
        }
    }

    /// Update data
    pub fn update_data(&mut self, data: WidgetData) {
        self.data = Some(data);
        self.last_refresh = Some(chrono::Utc::now());
        self.is_loading = false;
        self.error = None;
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    /// Set error
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.is_loading = false;
    }
}

/// User preferences for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred theme (light/dark)
    pub theme: String,

    /// Auto-refresh enabled
    pub auto_refresh: bool,

    /// Preferred page size for tables
    pub default_page_size: usize,

    /// Collapsed widget IDs
    pub collapsed_widgets: Vec<String>,

    /// Custom preferences
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            auto_refresh: true,
            default_page_size: 25,
            collapsed_widgets: Vec::new(),
            custom: HashMap::new(),
        }
    }
}

impl DashboardState {
    /// Create a new dashboard state
    pub fn new(config: DashboardConfig) -> Self {
        let layout = ResponsiveLayout::new(config.id.clone());

        Self {
            config,
            layout,
            widgets: HashMap::new(),
            preferences: UserPreferences::default(),
            last_updated: chrono::Utc::now(),
            version: 0,
        }
    }

    /// Add a widget
    pub fn add_widget(
        &mut self,
        widget_config: WidgetConfig,
        widget_layout: WidgetLayout,
    ) -> DashboardResult<()> {
        let widget_id = widget_config.metadata.id.clone();

        // Check widget limit
        if self.widgets.len() >= self.config.max_widgets as usize {
            return Err(DashboardError::validation(format!(
                "Maximum widget limit ({}) reached",
                self.config.max_widgets
            )));
        }

        // Add to layout
        self.layout.add_widget_responsive(widget_id.clone(), widget_layout)?;

        // Add to widget states
        self.widgets.insert(widget_id.clone(), WidgetState::new(widget_config));

        self.update_version();
        Ok(())
    }

    /// Remove a widget
    pub fn remove_widget(&mut self, widget_id: &str) -> DashboardResult<()> {
        self.layout.remove_widget(widget_id)?;
        self.widgets.remove(widget_id);
        self.update_version();
        Ok(())
    }

    /// Get widget state
    pub fn get_widget(&self, widget_id: &str) -> Option<&WidgetState> {
        self.widgets.get(widget_id)
    }

    /// Get mutable widget state
    pub fn get_widget_mut(&mut self, widget_id: &str) -> Option<&mut WidgetState> {
        self.widgets.get_mut(widget_id)
    }

    /// Update widget data
    pub fn update_widget_data(&mut self, widget_id: &str, data: WidgetData) -> DashboardResult<()> {
        let widget = self.widgets.get_mut(widget_id)
            .ok_or_else(|| DashboardError::not_found(format!("Widget not found: {}", widget_id)))?;

        widget.update_data(data);
        self.update_version();
        Ok(())
    }

    /// Set breakpoint
    pub fn set_breakpoint(&mut self, breakpoint: Breakpoint) {
        self.layout.set_breakpoint(breakpoint);
    }

    /// Get stale widgets
    pub fn get_stale_widgets(&self, ttl_seconds: u32) -> Vec<&str> {
        self.widgets
            .iter()
            .filter(|(_, state)| state.is_stale(ttl_seconds))
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Update version
    fn update_version(&mut self) {
        self.version += 1;
        self.last_updated = chrono::Utc::now();
    }

    /// Validate state
    pub fn validate(&self) -> DashboardResult<()> {
        self.config.validate_config()?;

        // Validate all widget IDs in layout exist in widgets
        for (_, layout) in &self.layout.breakpoint_layouts {
            for widget_layout in &layout.widgets {
                if !self.widgets.contains_key(&widget_layout.widget_id) {
                    return Err(DashboardError::validation(format!(
                        "Layout references non-existent widget: {}",
                        widget_layout.widget_id
                    )));
                }
            }
        }

        Ok(())
    }
}

/// Thread-safe dashboard state manager
pub struct DashboardStateManager {
    states: Arc<RwLock<HashMap<String, DashboardState>>>,
}

impl DashboardStateManager {
    /// Create a new state manager
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new dashboard
    pub async fn create_dashboard(&self, config: DashboardConfig) -> DashboardResult<String> {
        let dashboard_id = config.id.clone();
        let state = DashboardState::new(config);

        let mut states = self.states.write().await;
        states.insert(dashboard_id.clone(), state);

        Ok(dashboard_id)
    }

    /// Get dashboard state
    pub async fn get_dashboard(&self, dashboard_id: &str) -> DashboardResult<DashboardState> {
        let states = self.states.read().await;
        states
            .get(dashboard_id)
            .cloned()
            .ok_or_else(|| DashboardError::not_found(format!("Dashboard not found: {}", dashboard_id)))
    }

    /// Update dashboard state
    pub async fn update_dashboard(
        &self,
        dashboard_id: &str,
        updater: impl FnOnce(&mut DashboardState) -> DashboardResult<()>,
    ) -> DashboardResult<()> {
        let mut states = self.states.write().await;
        let state = states
            .get_mut(dashboard_id)
            .ok_or_else(|| DashboardError::not_found(format!("Dashboard not found: {}", dashboard_id)))?;

        updater(state)?;
        Ok(())
    }

    /// Delete dashboard
    pub async fn delete_dashboard(&self, dashboard_id: &str) -> DashboardResult<()> {
        let mut states = self.states.write().await;
        states
            .remove(dashboard_id)
            .ok_or_else(|| DashboardError::not_found(format!("Dashboard not found: {}", dashboard_id)))?;
        Ok(())
    }

    /// List all dashboard IDs
    pub async fn list_dashboards(&self) -> Vec<String> {
        let states = self.states.read().await;
        states.keys().cloned().collect()
    }

    /// Get dashboard count
    pub async fn count(&self) -> usize {
        let states = self.states.read().await;
        states.len()
    }
}

impl Default for DashboardStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::{WidgetMetadata, WidgetType, DisplayOptions, InteractionOptions};

    #[test]
    fn test_widget_state() {
        let metadata = WidgetMetadata::new("w1".to_string(), WidgetType::Metrics, "Test".to_string());
        let config = WidgetConfig {
            metadata,
            data_source: None,
            config: serde_json::json!({}),
            display: DisplayOptions::default(),
            interaction: InteractionOptions::default(),
        };

        let mut state = WidgetState::new(config);
        assert!(state.is_stale(60));

        let data = WidgetData::new(serde_json::json!({}));
        state.update_data(data);
        assert!(!state.is_stale(60));
    }

    #[test]
    fn test_dashboard_state() {
        let config = DashboardConfig::default();
        let state = DashboardState::new(config);

        assert_eq!(state.widgets.len(), 0);
        assert_eq!(state.version, 0);
    }

    #[tokio::test]
    async fn test_state_manager() {
        let manager = DashboardStateManager::new();
        let config = DashboardConfig::new("Test Dashboard".to_string());
        let dashboard_id = config.id.clone();

        manager.create_dashboard(config).await.unwrap();

        let state = manager.get_dashboard(&dashboard_id).await.unwrap();
        assert_eq!(state.config.name, "Test Dashboard");

        let dashboards = manager.list_dashboards().await;
        assert_eq!(dashboards.len(), 1);
    }
}
