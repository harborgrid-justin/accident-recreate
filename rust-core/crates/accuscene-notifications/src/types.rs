//! Core notification types and structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Notification severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationLevel {
    /// Informational message
    Info,
    /// Success message
    Success,
    /// Warning message
    Warning,
    /// Error message
    Error,
    /// Critical alert
    Alert,
}

impl NotificationLevel {
    /// Get priority score for this level
    pub fn priority(&self) -> u8 {
        match self {
            NotificationLevel::Alert => 5,
            NotificationLevel::Error => 4,
            NotificationLevel::Warning => 3,
            NotificationLevel::Success => 2,
            NotificationLevel::Info => 1,
        }
    }
}

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low = 1,
    Normal = 2,
    High = 3,
    Urgent = 4,
    Critical = 5,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// Notification category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationCategory {
    /// System notifications
    System,
    /// Case-related notifications
    Case,
    /// Collaboration notifications
    Collaboration,
    /// Analysis notifications
    Analysis,
    /// Report notifications
    Report,
    /// Security notifications
    Security,
    /// Billing notifications
    Billing,
    /// Marketing notifications
    Marketing,
    /// Custom category
    Custom(String),
}

/// Notification action button
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub id: String,
    pub label: String,
    pub url: Option<String>,
    pub action_type: ActionType,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Action type for notification buttons
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Navigate,
    ApiCall,
    Dismiss,
    Custom(String),
}

/// Core notification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique notification ID
    pub id: Uuid,

    /// User ID this notification is for
    pub user_id: String,

    /// Organization/tenant ID
    pub organization_id: Option<String>,

    /// Notification level
    pub level: NotificationLevel,

    /// Priority
    pub priority: Priority,

    /// Category
    pub category: NotificationCategory,

    /// Title/subject
    pub title: String,

    /// Message body
    pub message: String,

    /// Optional HTML message body
    pub html_message: Option<String>,

    /// Actions/buttons
    pub actions: Vec<NotificationAction>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Related entity ID (e.g., case_id, report_id)
    pub related_entity_id: Option<String>,

    /// Related entity type
    pub related_entity_type: Option<String>,

    /// Whether notification has been read
    pub read: bool,

    /// When notification was read
    pub read_at: Option<DateTime<Utc>>,

    /// Whether notification is archived
    pub archived: bool,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Expiration timestamp
    pub expires_at: Option<DateTime<Utc>>,

    /// Sender information
    pub sender: Option<NotificationSender>,

    /// Template ID if using template
    pub template_id: Option<String>,

    /// Template variables
    pub template_vars: HashMap<String, serde_json::Value>,
}

impl Notification {
    /// Create a new notification
    pub fn new(
        user_id: impl Into<String>,
        level: NotificationLevel,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: user_id.into(),
            organization_id: None,
            level,
            priority: Priority::default(),
            category: NotificationCategory::System,
            title: title.into(),
            message: message.into(),
            html_message: None,
            actions: Vec::new(),
            metadata: HashMap::new(),
            related_entity_id: None,
            related_entity_type: None,
            read: false,
            read_at: None,
            archived: false,
            created_at: Utc::now(),
            expires_at: None,
            sender: None,
            template_id: None,
            template_vars: HashMap::new(),
        }
    }

    /// Mark notification as read
    pub fn mark_read(&mut self) {
        self.read = true;
        self.read_at = Some(Utc::now());
    }

    /// Mark notification as unread
    pub fn mark_unread(&mut self) {
        self.read = false;
        self.read_at = None;
    }

    /// Check if notification is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Add an action button
    pub fn add_action(&mut self, action: NotificationAction) {
        self.actions.push(action);
    }

    /// Set metadata field
    pub fn set_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
    }
}

/// Notification sender information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSender {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub sender_type: SenderType,
}

/// Type of notification sender
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SenderType {
    User,
    System,
    Agent,
    Integration,
}

/// Notification delivery status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryStatus {
    pub notification_id: Uuid,
    pub channel: String,
    pub status: DeliveryState,
    pub attempts: u32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// State of notification delivery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryState {
    Pending,
    Processing,
    Delivered,
    Failed,
    Cancelled,
}

/// Bulk notification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkNotification {
    pub user_ids: Vec<String>,
    pub level: NotificationLevel,
    pub priority: Priority,
    pub category: NotificationCategory,
    pub title: String,
    pub message: String,
    pub template_id: Option<String>,
    pub template_vars: HashMap<String, serde_json::Value>,
}

/// Notification statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationStats {
    pub total: u64,
    pub unread: u64,
    pub by_level: HashMap<String, u64>,
    pub by_category: HashMap<String, u64>,
    pub by_channel: HashMap<String, u64>,
}

impl Default for NotificationStats {
    fn default() -> Self {
        Self {
            total: 0,
            unread: 0,
            by_level: HashMap::new(),
            by_category: HashMap::new(),
            by_channel: HashMap::new(),
        }
    }
}
