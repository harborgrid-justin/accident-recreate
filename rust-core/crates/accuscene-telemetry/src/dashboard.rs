//! Metrics dashboard data

use crate::{
    alerts::{Alert, AlertSeverity, AlertStatistics},
    events::Event,
    health::HealthStatus,
    performance::ProfileSummary,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dashboard data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    /// Dashboard metadata
    pub metadata: DashboardMetadata,

    /// System overview
    pub overview: SystemOverview,

    /// Metrics data
    pub metrics: MetricsData,

    /// Health status
    pub health: Option<HealthStatus>,

    /// Active alerts
    pub alerts: Vec<Alert>,

    /// Alert statistics
    pub alert_stats: Option<AlertStatistics>,

    /// Recent events
    pub recent_events: Vec<Event>,

    /// Performance data
    pub performance: Option<ProfileSummary>,

    /// Custom widgets
    pub widgets: Vec<Widget>,
}

impl Dashboard {
    /// Create a new dashboard
    pub fn new() -> Self {
        Self {
            metadata: DashboardMetadata {
                generated_at: Utc::now(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            overview: SystemOverview::default(),
            metrics: MetricsData::default(),
            health: None,
            alerts: Vec::new(),
            alert_stats: None,
            recent_events: Vec::new(),
            performance: None,
            widgets: Vec::new(),
        }
    }

    /// Update the dashboard with fresh data
    pub fn refresh(&mut self) {
        self.metadata.generated_at = Utc::now();
    }

    /// Set system overview
    pub fn set_overview(&mut self, overview: SystemOverview) {
        self.overview = overview;
    }

    /// Set metrics data
    pub fn set_metrics(&mut self, metrics: MetricsData) {
        self.metrics = metrics;
    }

    /// Set health status
    pub fn set_health(&mut self, health: HealthStatus) {
        self.health = Some(health);
    }

    /// Set alerts
    pub fn set_alerts(&mut self, alerts: Vec<Alert>, stats: AlertStatistics) {
        self.alerts = alerts;
        self.alert_stats = Some(stats);
    }

    /// Set recent events
    pub fn set_recent_events(&mut self, events: Vec<Event>) {
        self.recent_events = events;
    }

    /// Set performance data
    pub fn set_performance(&mut self, performance: ProfileSummary) {
        self.performance = Some(performance);
    }

    /// Add a widget
    pub fn add_widget(&mut self, widget: Widget) {
        self.widgets.push(widget);
    }

    /// Remove a widget
    pub fn remove_widget(&mut self, widget_id: &str) {
        self.widgets.retain(|w| w.id != widget_id);
    }

    /// Get a widget by ID
    pub fn get_widget(&self, widget_id: &str) -> Option<&Widget> {
        self.widgets.iter().find(|w| w.id == widget_id)
    }

    /// Export to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate a summary
    pub fn summary(&self) -> DashboardSummary {
        DashboardSummary {
            uptime_secs: self.overview.uptime_secs,
            total_requests: self.overview.total_requests,
            active_sessions: self.overview.active_sessions,
            error_rate: self.overview.error_rate,
            avg_response_time_ms: self.overview.avg_response_time_ms,
            health_status: self.health.as_ref().map(|h| h.is_healthy()).unwrap_or(false),
            active_alerts: self.alerts.iter().filter(|a| a.is_active()).count(),
            critical_alerts: self
                .alerts
                .iter()
                .filter(|a| a.is_active() && a.severity == AlertSeverity::Critical)
                .count(),
        }
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

/// Dashboard metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetadata {
    pub generated_at: DateTime<Utc>,
    pub version: String,
}

/// System overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverview {
    /// System uptime in seconds
    pub uptime_secs: u64,

    /// Total requests processed
    pub total_requests: u64,

    /// Active sessions
    pub active_sessions: u64,

    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,

    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,

    /// CPU usage percentage
    pub cpu_usage_percent: f64,

    /// Memory usage in bytes
    pub memory_usage_bytes: u64,

    /// Memory usage percentage
    pub memory_usage_percent: f64,

    /// Disk usage in bytes
    pub disk_usage_bytes: u64,

    /// Network bytes sent
    pub network_bytes_sent: u64,

    /// Network bytes received
    pub network_bytes_received: u64,
}

impl Default for SystemOverview {
    fn default() -> Self {
        Self {
            uptime_secs: 0,
            total_requests: 0,
            active_sessions: 0,
            error_rate: 0.0,
            avg_response_time_ms: 0.0,
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            memory_usage_percent: 0.0,
            disk_usage_bytes: 0,
            network_bytes_sent: 0,
            network_bytes_received: 0,
        }
    }
}

/// Metrics data for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    /// Simulation metrics
    pub simulation: SimulationMetrics,

    /// Database metrics
    pub database: DatabaseMetrics,

    /// Cache metrics
    pub cache: CacheMetrics,

    /// API metrics
    pub api: ApiMetrics,

    /// Custom metrics
    pub custom: HashMap<String, f64>,
}

impl Default for MetricsData {
    fn default() -> Self {
        Self {
            simulation: SimulationMetrics::default(),
            database: DatabaseMetrics::default(),
            cache: CacheMetrics::default(),
            api: ApiMetrics::default(),
            custom: HashMap::new(),
        }
    }
}

/// Simulation-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SimulationMetrics {
    /// Current FPS
    pub fps: f64,

    /// Average step time in milliseconds
    pub avg_step_time_ms: f64,

    /// Active simulations
    pub active_simulations: u64,

    /// Total simulations run
    pub total_simulations: u64,

    /// Total simulation time in seconds
    pub total_simulation_time_secs: f64,
}

/// Database-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseMetrics {
    /// Query latency in milliseconds
    pub query_latency_ms: f64,

    /// Queries per second
    pub queries_per_second: f64,

    /// Active connections
    pub active_connections: u64,

    /// Connection pool size
    pub pool_size: u64,

    /// Total queries executed
    pub total_queries: u64,

    /// Failed queries
    pub failed_queries: u64,
}

/// Cache-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheMetrics {
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,

    /// Cache miss rate (0.0 to 1.0)
    pub miss_rate: f64,

    /// Total hits
    pub total_hits: u64,

    /// Total misses
    pub total_misses: u64,

    /// Cached items
    pub cached_items: u64,

    /// Cache size in bytes
    pub cache_size_bytes: u64,
}

/// API-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiMetrics {
    /// Requests per second
    pub requests_per_second: f64,

    /// Average latency in milliseconds
    pub avg_latency_ms: f64,

    /// P95 latency in milliseconds
    pub p95_latency_ms: f64,

    /// P99 latency in milliseconds
    pub p99_latency_ms: f64,

    /// Total requests
    pub total_requests: u64,

    /// Failed requests
    pub failed_requests: u64,

    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

/// Dashboard widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    /// Widget ID
    pub id: String,

    /// Widget title
    pub title: String,

    /// Widget type
    pub widget_type: WidgetType,

    /// Widget data
    pub data: serde_json::Value,

    /// Widget position
    pub position: WidgetPosition,

    /// Widget size
    pub size: WidgetSize,
}

/// Widget type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetType {
    /// Line chart
    LineChart,
    /// Bar chart
    BarChart,
    /// Pie chart
    PieChart,
    /// Metric display
    Metric,
    /// Table
    Table,
    /// Status indicator
    Status,
    /// Custom widget
    Custom,
}

/// Widget position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
}

/// Widget size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
}

/// Dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub uptime_secs: u64,
    pub total_requests: u64,
    pub active_sessions: u64,
    pub error_rate: f64,
    pub avg_response_time_ms: f64,
    pub health_status: bool,
    pub active_alerts: usize,
    pub critical_alerts: usize,
}

/// Builder for creating dashboards
pub struct DashboardBuilder {
    dashboard: Dashboard,
}

impl DashboardBuilder {
    /// Create a new dashboard builder
    pub fn new() -> Self {
        Self {
            dashboard: Dashboard::new(),
        }
    }

    /// Set overview
    pub fn with_overview(mut self, overview: SystemOverview) -> Self {
        self.dashboard.set_overview(overview);
        self
    }

    /// Set metrics
    pub fn with_metrics(mut self, metrics: MetricsData) -> Self {
        self.dashboard.set_metrics(metrics);
        self
    }

    /// Set health
    pub fn with_health(mut self, health: HealthStatus) -> Self {
        self.dashboard.set_health(health);
        self
    }

    /// Add widget
    pub fn with_widget(mut self, widget: Widget) -> Self {
        self.dashboard.add_widget(widget);
        self
    }

    /// Build the dashboard
    pub fn build(self) -> Dashboard {
        self.dashboard
    }
}

impl Default for DashboardBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_creation() {
        let dashboard = Dashboard::new();
        assert_eq!(dashboard.alerts.len(), 0);
        assert_eq!(dashboard.widgets.len(), 0);
    }

    #[test]
    fn test_dashboard_builder() {
        let overview = SystemOverview {
            uptime_secs: 3600,
            total_requests: 1000,
            ..Default::default()
        };

        let dashboard = DashboardBuilder::new()
            .with_overview(overview)
            .build();

        assert_eq!(dashboard.overview.uptime_secs, 3600);
        assert_eq!(dashboard.overview.total_requests, 1000);
    }

    #[test]
    fn test_dashboard_summary() {
        let mut dashboard = Dashboard::new();
        dashboard.overview.uptime_secs = 1000;
        dashboard.overview.total_requests = 5000;

        let summary = dashboard.summary();
        assert_eq!(summary.uptime_secs, 1000);
        assert_eq!(summary.total_requests, 5000);
    }
}
