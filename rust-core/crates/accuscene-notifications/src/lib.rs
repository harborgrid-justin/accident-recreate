//! AccuScene Enterprise Notification System
//!
//! A comprehensive, enterprise-grade real-time notification system with:
//! - Multi-channel delivery (Email, SMS, Push, WebSocket, Webhooks)
//! - Priority-based dispatching
//! - User preferences and quiet hours
//! - Template engine with built-in templates
//! - Scheduled notifications with cron support
//! - Notification batching and aggregation
//! - Persistent storage and history
//! - WebSocket real-time updates
//!
//! # Example
//!
//! ```no_run
//! use accuscene_notifications::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Initialize notification system
//!     let system = NotificationSystem::new(NotificationConfig::default()).await?;
//!
//!     // Send a notification
//!     let notification = Notification::new(
//!         "user123",
//!         NotificationLevel::Info,
//!         "Welcome!",
//!         "Welcome to AccuScene Enterprise"
//!     );
//!
//!     system.send(notification, vec!["in_app".to_string()]).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod aggregator;
pub mod channel;
pub mod config;
pub mod dispatcher;
pub mod error;
pub mod preferences;
pub mod scheduler;
pub mod store;
pub mod templates;
pub mod types;

// Re-exports
pub use aggregator::{AggregationRule, NotificationAggregator, NotificationBatch};
pub use channel::{Channel, ChannelRegistry, EmailChannel, InAppChannel, PushChannel, SmsChannel, WebhookChannel};
pub use config::{NotificationConfig, EmailConfig, SmsConfig, PushConfig, WebhookConfig};
pub use dispatcher::{NotificationDispatcher, DispatcherStats};
pub use error::{NotificationError, Result};
pub use preferences::{NotificationPreferences, PreferenceManager, QuietHours};
pub use scheduler::{NotificationScheduler, ScheduledNotification, SchedulerStats};
pub use store::NotificationStore;
pub use templates::{NotificationTemplate, TemplateEngine};
pub use types::{
    Notification, NotificationAction, NotificationCategory, NotificationLevel,
    NotificationStats, Priority, DeliveryStatus, DeliveryState
};

use sqlx::PgPool;
use std::sync::Arc;

/// Main notification system
pub struct NotificationSystem {
    config: NotificationConfig,
    store: Arc<NotificationStore>,
    preference_manager: Arc<PreferenceManager>,
    channel_registry: Arc<ChannelRegistry>,
    dispatcher: Arc<NotificationDispatcher>,
    scheduler: Arc<NotificationScheduler>,
    aggregator: Arc<NotificationAggregator>,
    template_engine: Arc<TemplateEngine>,
}

impl NotificationSystem {
    /// Create a new notification system
    pub async fn new(config: NotificationConfig) -> Result<Self> {
        // Validate configuration
        config.validate()
            .map_err(|e| NotificationError::Configuration(e))?;

        // Initialize database pool
        let pool = PgPool::connect(&config.database.url).await?;

        // Initialize store
        let store = Arc::new(NotificationStore::new(pool.clone()));
        store.initialize().await?;

        // Initialize preference manager
        let preference_manager = Arc::new(PreferenceManager::new(pool.clone()));
        preference_manager.initialize().await?;

        // Initialize channel registry
        let mut channel_registry = ChannelRegistry::new();
        channel_registry.register(Arc::new(EmailChannel::new(config.channels.email.clone())));
        channel_registry.register(Arc::new(SmsChannel::new(config.channels.sms.clone())));
        channel_registry.register(Arc::new(PushChannel::new(config.channels.push.clone())));
        channel_registry.register(Arc::new(WebhookChannel::new(config.channels.webhook.clone())));
        channel_registry.register(Arc::new(InAppChannel::new(config.channels.in_app.enabled)));
        let channel_registry = Arc::new(channel_registry);

        // Initialize dispatcher
        let dispatcher = Arc::new(NotificationDispatcher::new(
            config.dispatcher.clone(),
            Arc::clone(&channel_registry),
            Arc::clone(&preference_manager),
        ));

        // Initialize scheduler
        let scheduler = Arc::new(NotificationScheduler::new(Arc::clone(&dispatcher)));

        // Initialize aggregator
        let aggregator = Arc::new(NotificationAggregator::new(Arc::clone(&dispatcher)));

        // Initialize template engine
        let template_engine = Arc::new(TemplateEngine::new(Some(&config.templates.templates_dir))?);
        template_engine.register_builtin_templates().await?;

        Ok(Self {
            config,
            store,
            preference_manager,
            channel_registry,
            dispatcher,
            scheduler,
            aggregator,
            template_engine,
        })
    }

    /// Start the notification system
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting AccuScene Notification System v0.2.5");

        // Start dispatcher
        Arc::get_mut(&mut self.dispatcher)
            .ok_or_else(|| NotificationError::Internal("Cannot start dispatcher".to_string()))?
            .start()
            .await?;

        // Start scheduler
        Arc::get_mut(&mut self.scheduler)
            .ok_or_else(|| NotificationError::Internal("Cannot start scheduler".to_string()))?
            .start()
            .await?;

        // Start aggregator
        Arc::get_mut(&mut self.aggregator)
            .ok_or_else(|| NotificationError::Internal("Cannot start aggregator".to_string()))?
            .start()
            .await?;

        // Register default aggregation rules
        self.aggregator.register_default_rules().await?;

        tracing::info!("Notification system started successfully");
        Ok(())
    }

    /// Send a notification
    pub async fn send(&self, notification: Notification, channels: Vec<String>) -> Result<uuid::Uuid> {
        // Save to store
        self.store.save(&notification).await?;

        // Check if should aggregate
        let aggregated = self.aggregator.add_notification(notification.clone()).await?;

        if !aggregated {
            // Dispatch immediately
            self.dispatcher.enqueue(notification.clone(), channels).await?;
        }

        Ok(notification.id)
    }

    /// Send a notification using a template
    pub async fn send_from_template(
        &self,
        user_id: String,
        template_id: &str,
        template_vars: std::collections::HashMap<String, serde_json::Value>,
        level: NotificationLevel,
        channels: Vec<String>,
    ) -> Result<uuid::Uuid> {
        let mut notification = Notification::new(user_id, level, "", "");
        notification.template_vars = template_vars;

        self.template_engine
            .apply_template(&mut notification, template_id)
            .await?;

        self.send(notification, channels).await
    }

    /// Send bulk notifications
    pub async fn send_bulk(&self, notifications: Vec<Notification>, channels: Vec<String>) -> Result<Vec<uuid::Uuid>> {
        let mut ids = Vec::new();

        for notification in notifications {
            match self.send(notification, channels.clone()).await {
                Ok(id) => ids.push(id),
                Err(e) => tracing::error!("Failed to send notification: {}", e),
            }
        }

        Ok(ids)
    }

    /// Get notification by ID
    pub async fn get(&self, id: uuid::Uuid) -> Result<Option<Notification>> {
        self.store.get(id).await
    }

    /// Get notifications for a user
    pub async fn get_for_user(&self, user_id: &str, limit: i64, offset: i64) -> Result<Vec<Notification>> {
        self.store.get_for_user(user_id, limit, offset).await
    }

    /// Get unread notifications
    pub async fn get_unread(&self, user_id: &str, limit: i64) -> Result<Vec<Notification>> {
        self.store.get_unread(user_id, limit).await
    }

    /// Mark notification as read
    pub async fn mark_read(&self, id: uuid::Uuid) -> Result<()> {
        self.store.mark_read(id).await
    }

    /// Mark all as read
    pub async fn mark_all_read(&self, user_id: &str) -> Result<u64> {
        self.store.mark_all_read(user_id).await
    }

    /// Archive notification
    pub async fn archive(&self, id: uuid::Uuid) -> Result<()> {
        self.store.archive(id).await
    }

    /// Delete notification
    pub async fn delete(&self, id: uuid::Uuid) -> Result<()> {
        self.store.delete(id).await
    }

    /// Get notification statistics
    pub async fn get_stats(&self, user_id: &str) -> Result<NotificationStats> {
        self.store.get_stats(user_id).await
    }

    /// Get user preferences
    pub async fn get_preferences(&self, user_id: &str) -> Result<NotificationPreferences> {
        self.preference_manager.get(user_id).await
    }

    /// Save user preferences
    pub async fn save_preferences(&self, preferences: &NotificationPreferences) -> Result<()> {
        self.preference_manager.save(preferences).await
    }

    /// Schedule a notification
    pub async fn schedule(
        &self,
        notification: Notification,
        cron_expression: String,
        channels: Vec<String>,
    ) -> Result<uuid::Uuid> {
        self.scheduler
            .schedule(notification, cron_expression, channels)
            .await
    }

    /// Get template engine
    pub fn template_engine(&self) -> &Arc<TemplateEngine> {
        &self.template_engine
    }

    /// Get store
    pub fn store(&self) -> &Arc<NotificationStore> {
        &self.store
    }

    /// Get preference manager
    pub fn preference_manager(&self) -> &Arc<PreferenceManager> {
        &self.preference_manager
    }

    /// Get dispatcher
    pub fn dispatcher(&self) -> &Arc<NotificationDispatcher> {
        &self.dispatcher
    }

    /// Get scheduler
    pub fn scheduler(&self) -> &Arc<NotificationScheduler> {
        &self.scheduler
    }

    /// Get aggregator
    pub fn aggregator(&self) -> &Arc<NotificationAggregator> {
        &self.aggregator
    }

    /// Shutdown the system
    pub async fn shutdown(&self) {
        tracing::info!("Shutting down notification system");
        self.dispatcher.shutdown().await;
        self.scheduler.shutdown().await;
        self.aggregator.shutdown().await;
    }

    /// Get system configuration
    pub fn config(&self) -> &NotificationConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_notification_creation() {
        let notif = Notification::new("user1", NotificationLevel::Info, "Test", "Test message");
        assert_eq!(notif.user_id, "user1");
        assert_eq!(notif.title, "Test");
        assert_eq!(notif.message, "Test message");
        assert!(!notif.read);
    }
}
