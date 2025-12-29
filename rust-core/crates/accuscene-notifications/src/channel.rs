//! Notification delivery channels

use crate::config::{EmailConfig, PushConfig, SmsConfig, WebhookConfig};
use crate::error::{NotificationError, Result};
use crate::types::{DeliveryState, DeliveryStatus, Notification};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Trait for notification delivery channels
#[async_trait]
pub trait Channel: Send + Sync {
    /// Channel name
    fn name(&self) -> &str;

    /// Deliver a notification through this channel
    async fn deliver(&self, notification: &Notification) -> Result<DeliveryStatus>;

    /// Check if channel supports this notification
    fn supports(&self, notification: &Notification) -> bool;

    /// Check if channel is enabled
    fn is_enabled(&self) -> bool;
}

/// Email delivery channel
pub struct EmailChannel {
    config: EmailConfig,
    client: Option<lettre::AsyncSmtpTransport<lettre::Tokio1Executor>>,
}

impl EmailChannel {
    pub fn new(config: EmailConfig) -> Self {
        let client = if config.enabled {
            use lettre::transport::smtp::authentication::Credentials;
            use lettre::AsyncSmtpTransport;

            let creds = Credentials::new(
                config.smtp_username.clone(),
                config.smtp_password.clone(),
            );

            let transport = if config.use_tls {
                AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&config.smtp_host)
                    .unwrap()
                    .credentials(creds)
                    .port(config.smtp_port)
                    .build()
            } else {
                AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(&config.smtp_host)
                    .credentials(creds)
                    .port(config.smtp_port)
                    .build()
            };

            Some(transport)
        } else {
            None
        };

        Self { config, client }
    }
}

#[async_trait]
impl Channel for EmailChannel {
    fn name(&self) -> &str {
        "email"
    }

    async fn deliver(&self, notification: &Notification) -> Result<DeliveryStatus> {
        use lettre::{Message, AsyncTransport};

        let mut status = DeliveryStatus {
            notification_id: notification.id,
            channel: self.name().to_string(),
            status: DeliveryState::Processing,
            attempts: 1,
            last_attempt_at: Some(Utc::now()),
            delivered_at: None,
            error_message: None,
        };

        if !self.is_enabled() {
            status.status = DeliveryState::Failed;
            status.error_message = Some("Email channel is disabled".to_string());
            return Ok(status);
        }

        let client = self.client.as_ref().ok_or_else(|| {
            NotificationError::EmailDelivery("Email client not initialized".to_string())
        })?;

        // Get user email from metadata
        let to_email = notification
            .metadata
            .get("email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NotificationError::EmailDelivery("No email in metadata".to_string()))?;

        // Build email
        let email = Message::builder()
            .from(
                format!("{} <{}>", self.config.from_name, self.config.from_address)
                    .parse()
                    .map_err(|e| NotificationError::EmailDelivery(format!("Invalid from: {}", e)))?,
            )
            .to(to_email
                .parse()
                .map_err(|e| NotificationError::EmailDelivery(format!("Invalid to: {}", e)))?)
            .subject(&notification.title)
            .body(notification.html_message.clone().unwrap_or_else(|| notification.message.clone()))
            .map_err(|e| NotificationError::EmailDelivery(format!("Failed to build email: {}", e)))?;

        // Send email
        match client.send(email).await {
            Ok(_) => {
                status.status = DeliveryState::Delivered;
                status.delivered_at = Some(Utc::now());
            }
            Err(e) => {
                status.status = DeliveryState::Failed;
                status.error_message = Some(format!("Failed to send email: {}", e));
            }
        }

        Ok(status)
    }

    fn supports(&self, notification: &Notification) -> bool {
        notification.metadata.contains_key("email")
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// SMS delivery channel
pub struct SmsChannel {
    config: SmsConfig,
    client: Option<reqwest::Client>,
}

impl SmsChannel {
    pub fn new(config: SmsConfig) -> Self {
        let client = if config.enabled {
            Some(reqwest::Client::new())
        } else {
            None
        };

        Self { config, client }
    }
}

#[async_trait]
impl Channel for SmsChannel {
    fn name(&self) -> &str {
        "sms"
    }

    async fn deliver(&self, notification: &Notification) -> Result<DeliveryStatus> {
        let mut status = DeliveryStatus {
            notification_id: notification.id,
            channel: self.name().to_string(),
            status: DeliveryState::Processing,
            attempts: 1,
            last_attempt_at: Some(Utc::now()),
            delivered_at: None,
            error_message: None,
        };

        if !self.is_enabled() {
            status.status = DeliveryState::Failed;
            status.error_message = Some("SMS channel is disabled".to_string());
            return Ok(status);
        }

        let phone = notification
            .metadata
            .get("phone")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NotificationError::SmsDelivery("No phone in metadata".to_string()))?;

        // In a real implementation, integrate with Twilio, AWS SNS, etc.
        tracing::info!(
            "Sending SMS to {} via {:?}: {}",
            phone,
            self.config.provider,
            notification.message
        );

        status.status = DeliveryState::Delivered;
        status.delivered_at = Some(Utc::now());

        Ok(status)
    }

    fn supports(&self, notification: &Notification) -> bool {
        notification.metadata.contains_key("phone")
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Push notification channel
pub struct PushChannel {
    config: PushConfig,
    client: Option<reqwest::Client>,
}

impl PushChannel {
    pub fn new(config: PushConfig) -> Self {
        let client = if config.enabled {
            Some(reqwest::Client::new())
        } else {
            None
        };

        Self { config, client }
    }
}

#[async_trait]
impl Channel for PushChannel {
    fn name(&self) -> &str {
        "push"
    }

    async fn deliver(&self, notification: &Notification) -> Result<DeliveryStatus> {
        let mut status = DeliveryStatus {
            notification_id: notification.id,
            channel: self.name().to_string(),
            status: DeliveryState::Processing,
            attempts: 1,
            last_attempt_at: Some(Utc::now()),
            delivered_at: None,
            error_message: None,
        };

        if !self.is_enabled() {
            status.status = DeliveryState::Failed;
            status.error_message = Some("Push channel is disabled".to_string());
            return Ok(status);
        }

        let device_token = notification
            .metadata
            .get("device_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                NotificationError::PushNotification("No device_token in metadata".to_string())
            })?;

        // In a real implementation, integrate with FCM/APNS
        tracing::info!(
            "Sending push notification to device {}: {}",
            device_token,
            notification.title
        );

        status.status = DeliveryState::Delivered;
        status.delivered_at = Some(Utc::now());

        Ok(status)
    }

    fn supports(&self, notification: &Notification) -> bool {
        notification.metadata.contains_key("device_token")
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Webhook delivery channel
pub struct WebhookChannel {
    config: WebhookConfig,
    client: reqwest::Client,
}

impl WebhookChannel {
    pub fn new(config: WebhookConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .unwrap();

        Self { config, client }
    }
}

#[async_trait]
impl Channel for WebhookChannel {
    fn name(&self) -> &str {
        "webhook"
    }

    async fn deliver(&self, notification: &Notification) -> Result<DeliveryStatus> {
        let mut status = DeliveryStatus {
            notification_id: notification.id,
            channel: self.name().to_string(),
            status: DeliveryState::Processing,
            attempts: 1,
            last_attempt_at: Some(Utc::now()),
            delivered_at: None,
            error_message: None,
        };

        if !self.is_enabled() {
            status.status = DeliveryState::Failed;
            status.error_message = Some("Webhook channel is disabled".to_string());
            return Ok(status);
        }

        let webhook_url = notification
            .metadata
            .get("webhook_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                NotificationError::WebhookDelivery("No webhook_url in metadata".to_string())
            })?;

        // Prepare webhook payload
        #[derive(Serialize)]
        struct WebhookPayload<'a> {
            notification_id: uuid::Uuid,
            user_id: &'a str,
            level: &'a str,
            title: &'a str,
            message: &'a str,
            timestamp: chrono::DateTime<Utc>,
        }

        let payload = WebhookPayload {
            notification_id: notification.id,
            user_id: &notification.user_id,
            level: &format!("{:?}", notification.level),
            title: &notification.title,
            message: &notification.message,
            timestamp: notification.created_at,
        };

        // Send webhook
        match self.client.post(webhook_url).json(&payload).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    status.status = DeliveryState::Delivered;
                    status.delivered_at = Some(Utc::now());
                } else {
                    status.status = DeliveryState::Failed;
                    status.error_message =
                        Some(format!("Webhook returned status: {}", response.status()));
                }
            }
            Err(e) => {
                status.status = DeliveryState::Failed;
                status.error_message = Some(format!("Failed to send webhook: {}", e));
            }
        }

        Ok(status)
    }

    fn supports(&self, notification: &Notification) -> bool {
        notification.metadata.contains_key("webhook_url")
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// In-app notification channel (stores in database)
pub struct InAppChannel {
    enabled: bool,
}

impl InAppChannel {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[async_trait]
impl Channel for InAppChannel {
    fn name(&self) -> &str {
        "in_app"
    }

    async fn deliver(&self, notification: &Notification) -> Result<DeliveryStatus> {
        let mut status = DeliveryStatus {
            notification_id: notification.id,
            channel: self.name().to_string(),
            status: DeliveryState::Processing,
            attempts: 1,
            last_attempt_at: Some(Utc::now()),
            delivered_at: None,
            error_message: None,
        };

        if !self.is_enabled() {
            status.status = DeliveryState::Failed;
            status.error_message = Some("In-app channel is disabled".to_string());
            return Ok(status);
        }

        // In-app notifications are handled by the store
        status.status = DeliveryState::Delivered;
        status.delivered_at = Some(Utc::now());

        Ok(status)
    }

    fn supports(&self, _notification: &Notification) -> bool {
        true // All notifications support in-app
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Channel registry
pub struct ChannelRegistry {
    channels: Vec<Arc<dyn Channel>>,
}

impl ChannelRegistry {
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
        }
    }

    /// Register a channel
    pub fn register(&mut self, channel: Arc<dyn Channel>) {
        self.channels.push(channel);
    }

    /// Get all enabled channels that support the notification
    pub fn get_channels_for(&self, notification: &Notification) -> Vec<Arc<dyn Channel>> {
        self.channels
            .iter()
            .filter(|c| c.is_enabled() && c.supports(notification))
            .cloned()
            .collect()
    }

    /// Get channel by name
    pub fn get_channel(&self, name: &str) -> Option<Arc<dyn Channel>> {
        self.channels
            .iter()
            .find(|c| c.name() == name)
            .cloned()
    }

    /// Get all channels
    pub fn all_channels(&self) -> &[Arc<dyn Channel>] {
        &self.channels
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
