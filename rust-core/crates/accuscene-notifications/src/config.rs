//! Notification system configuration

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main notification system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Database configuration
    pub database: DatabaseConfig,

    /// Message queue configuration
    pub queue: QueueConfig,

    /// Channel configurations
    pub channels: ChannelConfigs,

    /// WebSocket configuration
    pub websocket: WebSocketConfig,

    /// Dispatcher configuration
    pub dispatcher: DispatcherConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Template configuration
    pub templates: TemplateConfig,

    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            queue: QueueConfig::default(),
            channels: ChannelConfigs::default(),
            websocket: WebSocketConfig::default(),
            dispatcher: DispatcherConfig::default(),
            storage: StorageConfig::default(),
            templates: TemplateConfig::default(),
            rate_limiting: RateLimitConfig::default(),
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://localhost/accuscene_notifications".to_string(),
            max_connections: 20,
            min_connections: 5,
            connection_timeout: 30,
            idle_timeout: 600,
        }
    }
}

/// Message queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    pub url: String,
    pub queue_name: String,
    pub exchange_name: String,
    pub prefetch_count: u16,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            url: "amqp://localhost:5672".to_string(),
            queue_name: "notifications".to_string(),
            exchange_name: "notifications_exchange".to_string(),
            prefetch_count: 100,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Channel configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfigs {
    pub email: EmailConfig,
    pub sms: SmsConfig,
    pub push: PushConfig,
    pub webhook: WebhookConfig,
    pub in_app: InAppConfig,
}

impl Default for ChannelConfigs {
    fn default() -> Self {
        Self {
            email: EmailConfig::default(),
            sms: SmsConfig::default(),
            push: PushConfig::default(),
            webhook: WebhookConfig::default(),
            in_app: InAppConfig::default(),
        }
    }
}

/// Email channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_address: String,
    pub from_name: String,
    pub use_tls: bool,
    pub max_retries: u32,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            smtp_username: "notifications@accuscene.com".to_string(),
            smtp_password: String::new(),
            from_address: "notifications@accuscene.com".to_string(),
            from_name: "AccuScene Notifications".to_string(),
            use_tls: true,
            max_retries: 3,
        }
    }
}

/// SMS channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    pub enabled: bool,
    pub provider: SmsProvider,
    pub api_key: String,
    pub api_secret: String,
    pub from_number: String,
    pub max_retries: u32,
}

impl Default for SmsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: SmsProvider::Twilio,
            api_key: String::new(),
            api_secret: String::new(),
            from_number: String::new(),
            max_retries: 3,
        }
    }
}

/// SMS provider
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SmsProvider {
    Twilio,
    Sns,
    Nexmo,
}

/// Push notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushConfig {
    pub enabled: bool,
    pub fcm_server_key: String,
    pub apns_cert_path: String,
    pub apns_key_path: String,
    pub max_retries: u32,
}

impl Default for PushConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            fcm_server_key: String::new(),
            apns_cert_path: String::new(),
            apns_key_path: String::new(),
            max_retries: 3,
        }
    }
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: 5000,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// In-app notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InAppConfig {
    pub enabled: bool,
    pub max_stored_per_user: u32,
    pub default_ttl_days: u32,
}

impl Default for InAppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_stored_per_user: 1000,
            default_ttl_days: 30,
        }
    }
}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub heartbeat_interval_ms: u64,
    pub client_timeout_ms: u64,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "0.0.0.0".to_string(),
            port: 8080,
            path: "/ws/notifications".to_string(),
            heartbeat_interval_ms: 30000,
            client_timeout_ms: 60000,
        }
    }
}

/// Dispatcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatcherConfig {
    pub worker_count: usize,
    pub queue_capacity: usize,
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
    pub priority_levels: u8,
}

impl Default for DispatcherConfig {
    fn default() -> Self {
        Self {
            worker_count: 10,
            queue_capacity: 10000,
            batch_size: 100,
            batch_timeout_ms: 1000,
            priority_levels: 5,
        }
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub retention_days: u32,
    pub cleanup_interval_hours: u32,
    pub archive_enabled: bool,
    pub archive_after_days: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            retention_days: 90,
            cleanup_interval_hours: 24,
            archive_enabled: true,
            archive_after_days: 30,
        }
    }
}

/// Template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub templates_dir: String,
    pub cache_enabled: bool,
    pub cache_size: usize,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            templates_dir: "./templates".to_string(),
            cache_enabled: true,
            cache_size: 100,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub max_per_user_per_minute: u32,
    pub max_per_user_per_hour: u32,
    pub max_per_user_per_day: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_per_user_per_minute: 10,
            max_per_user_per_hour: 100,
            max_per_user_per_day: 1000,
        }
    }
}

impl NotificationConfig {
    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = config::Config::builder()
            .add_source(config::File::with_name(path))
            .add_source(config::Environment::with_prefix("ACCUSCENE_NOTIFICATIONS"))
            .build()?;

        Ok(config.try_deserialize()?)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.database.max_connections < self.database.min_connections {
            return Err("max_connections must be >= min_connections".to_string());
        }

        if self.dispatcher.worker_count == 0 {
            return Err("worker_count must be > 0".to_string());
        }

        if self.dispatcher.queue_capacity == 0 {
            return Err("queue_capacity must be > 0".to_string());
        }

        Ok(())
    }
}
