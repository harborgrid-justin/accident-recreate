//! Widget system module
//!
//! Provides the core widget abstraction and common functionality

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

use crate::error::{WidgetError, WidgetResult};

pub mod charts;
pub mod metrics;
pub mod tables;

/// Widget type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WidgetType {
    /// Metrics display widget
    Metrics,
    /// Chart widget (line, bar, pie, etc.)
    Chart,
    /// Data table widget
    Table,
    /// Text/markdown widget
    Text,
    /// Image widget
    Image,
    /// Custom widget
    Custom,
}

impl WidgetType {
    /// Get widget type from string
    pub fn from_str(s: &str) -> WidgetResult<Self> {
        match s.to_lowercase().as_str() {
            "metrics" => Ok(WidgetType::Metrics),
            "chart" => Ok(WidgetType::Chart),
            "table" => Ok(WidgetType::Table),
            "text" => Ok(WidgetType::Text),
            "image" => Ok(WidgetType::Image),
            "custom" => Ok(WidgetType::Custom),
            _ => Err(WidgetError::invalid_type(s)),
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            WidgetType::Metrics => "metrics",
            WidgetType::Chart => "chart",
            WidgetType::Table => "table",
            WidgetType::Text => "text",
            WidgetType::Image => "image",
            WidgetType::Custom => "custom",
        }
    }
}

/// Widget refresh strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefreshStrategy {
    /// Manual refresh only
    Manual,
    /// Auto-refresh with interval
    Interval { seconds: u32 },
    /// Real-time updates
    Realtime,
    /// On-demand (lazy load)
    OnDemand,
}

/// Widget data source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    /// Data source type (api, websocket, static, etc.)
    pub source_type: String,

    /// Endpoint or resource identifier
    pub endpoint: String,

    /// Query parameters
    pub params: HashMap<String, String>,

    /// Refresh strategy
    pub refresh: RefreshStrategy,

    /// Authentication required
    pub requires_auth: bool,

    /// Cache TTL in seconds
    pub cache_ttl: Option<u32>,
}

impl DataSource {
    /// Create a new data source
    pub fn new(source_type: String, endpoint: String) -> Self {
        Self {
            source_type,
            endpoint,
            params: HashMap::new(),
            refresh: RefreshStrategy::Interval { seconds: 30 },
            requires_auth: true,
            cache_ttl: Some(60),
        }
    }

    /// Create an API data source
    pub fn api(endpoint: String) -> Self {
        Self::new("api".to_string(), endpoint)
    }

    /// Create a WebSocket data source
    pub fn websocket(endpoint: String) -> Self {
        let mut source = Self::new("websocket".to_string(), endpoint);
        source.refresh = RefreshStrategy::Realtime;
        source.cache_ttl = None;
        source
    }

    /// Add query parameter
    pub fn with_param(mut self, key: String, value: String) -> Self {
        self.params.insert(key, value);
        self
    }

    /// Set refresh strategy
    pub fn with_refresh(mut self, refresh: RefreshStrategy) -> Self {
        self.refresh = refresh;
        self
    }
}

/// Widget metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetMetadata {
    /// Widget unique identifier
    pub id: String,

    /// Widget type
    pub widget_type: WidgetType,

    /// Widget title
    pub title: String,

    /// Widget description
    pub description: Option<String>,

    /// Widget tags for categorization
    pub tags: Vec<String>,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Version for optimistic locking
    pub version: u64,
}

impl WidgetMetadata {
    /// Create new widget metadata
    pub fn new(id: String, widget_type: WidgetType, title: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            widget_type,
            title,
            description: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            version: 0,
        }
    }
}

/// Base widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    /// Widget metadata
    pub metadata: WidgetMetadata,

    /// Data source
    pub data_source: Option<DataSource>,

    /// Widget-specific configuration
    pub config: JsonValue,

    /// Display options
    pub display: DisplayOptions,

    /// Interaction options
    pub interaction: InteractionOptions,
}

/// Display options for widgets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayOptions {
    /// Show title
    pub show_title: bool,

    /// Show border
    pub show_border: bool,

    /// Background color (hex)
    pub background_color: Option<String>,

    /// Text color (hex)
    pub text_color: Option<String>,

    /// Padding in pixels
    pub padding: u32,

    /// Custom CSS classes
    pub css_classes: Vec<String>,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            show_title: true,
            show_border: true,
            background_color: None,
            text_color: None,
            padding: 16,
            css_classes: Vec::new(),
        }
    }
}

/// Interaction options for widgets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionOptions {
    /// Enable click events
    pub clickable: bool,

    /// Enable hover effects
    pub hoverable: bool,

    /// Enable drill-down
    pub drilldown_enabled: bool,

    /// Enable export
    pub export_enabled: bool,

    /// Enable fullscreen
    pub fullscreen_enabled: bool,
}

impl Default for InteractionOptions {
    fn default() -> Self {
        Self {
            clickable: false,
            hoverable: true,
            drilldown_enabled: false,
            export_enabled: true,
            fullscreen_enabled: true,
        }
    }
}

/// Widget data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetData {
    /// Data payload
    pub data: JsonValue,

    /// Timestamp of data
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Data is stale
    pub is_stale: bool,

    /// Error message if data fetch failed
    pub error: Option<String>,

    /// Metadata about the data
    pub metadata: HashMap<String, String>,
}

impl WidgetData {
    /// Create new widget data
    pub fn new(data: JsonValue) -> Self {
        Self {
            data,
            timestamp: chrono::Utc::now(),
            is_stale: false,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// Create error data
    pub fn error(message: String) -> Self {
        Self {
            data: JsonValue::Null,
            timestamp: chrono::Utc::now(),
            is_stale: true,
            error: Some(message),
            metadata: HashMap::new(),
        }
    }

    /// Check if data is stale based on TTL
    pub fn is_stale_with_ttl(&self, ttl_seconds: u32) -> bool {
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(self.timestamp);
        age.num_seconds() as u32 > ttl_seconds
    }
}

/// Widget trait for common functionality
#[async_trait]
pub trait Widget: Send + Sync {
    /// Get widget configuration
    fn config(&self) -> &WidgetConfig;

    /// Get widget data
    fn data(&self) -> Option<&WidgetData>;

    /// Fetch fresh data
    async fn fetch_data(&mut self) -> WidgetResult<WidgetData>;

    /// Validate widget configuration
    fn validate(&self) -> WidgetResult<()>;

    /// Handle user interaction
    async fn handle_interaction(&mut self, event: InteractionEvent) -> WidgetResult<()> {
        // Default implementation - no-op
        Ok(())
    }

    /// Export widget data
    fn export(&self, format: ExportFormat) -> WidgetResult<Vec<u8>> {
        Err(WidgetError::render_error("Export not supported for this widget type"))
    }
}

/// User interaction event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionEvent {
    /// Event type
    pub event_type: String,

    /// Event data
    pub data: JsonValue,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// PNG image
    Png,
    /// PDF document
    Pdf,
}

impl ExportFormat {
    /// Get file extension
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
            ExportFormat::Png => "png",
            ExportFormat::Pdf => "pdf",
        }
    }

    /// Get MIME type
    pub fn mime_type(&self) -> &'static str {
        match self {
            ExportFormat::Json => "application/json",
            ExportFormat::Csv => "text/csv",
            ExportFormat::Png => "image/png",
            ExportFormat::Pdf => "application/pdf",
        }
    }
}

/// Widget registry for managing widget instances
#[derive(Debug, Default)]
pub struct WidgetRegistry {
    widgets: HashMap<String, Box<dyn Widget>>,
}

impl WidgetRegistry {
    /// Create a new widget registry
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
        }
    }

    /// Register a widget
    pub fn register(&mut self, id: String, widget: Box<dyn Widget>) {
        self.widgets.insert(id, widget);
    }

    /// Get a widget by ID
    pub fn get(&self, id: &str) -> Option<&dyn Widget> {
        self.widgets.get(id).map(|w| w.as_ref())
    }

    /// Get a mutable widget by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut dyn Widget> {
        self.widgets.get_mut(id).map(|w| w.as_mut())
    }

    /// Remove a widget
    pub fn remove(&mut self, id: &str) -> Option<Box<dyn Widget>> {
        self.widgets.remove(id)
    }

    /// Get all widget IDs
    pub fn widget_ids(&self) -> Vec<String> {
        self.widgets.keys().cloned().collect()
    }

    /// Get widget count
    pub fn count(&self) -> usize {
        self.widgets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_type_from_str() {
        assert_eq!(WidgetType::from_str("metrics").unwrap(), WidgetType::Metrics);
        assert_eq!(WidgetType::from_str("CHART").unwrap(), WidgetType::Chart);
        assert!(WidgetType::from_str("invalid").is_err());
    }

    #[test]
    fn test_data_source_builder() {
        let source = DataSource::api("/api/metrics".to_string())
            .with_param("limit".to_string(), "100".to_string())
            .with_refresh(RefreshStrategy::Manual);

        assert_eq!(source.source_type, "api");
        assert_eq!(source.params.get("limit").unwrap(), "100");
        assert_eq!(source.refresh, RefreshStrategy::Manual);
    }

    #[test]
    fn test_widget_data_staleness() {
        let data = WidgetData::new(JsonValue::Null);
        assert!(!data.is_stale_with_ttl(100));

        let mut old_data = data.clone();
        old_data.timestamp = chrono::Utc::now() - chrono::Duration::seconds(200);
        assert!(old_data.is_stale_with_ttl(100));
    }
}
