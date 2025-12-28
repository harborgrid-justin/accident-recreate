//! Notification batching and aggregation

use crate::dispatcher::NotificationDispatcher;
use crate::error::{NotificationError, Result};
use crate::types::{Notification, NotificationCategory, NotificationLevel, Priority};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Aggregation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: Option<NotificationCategory>,
    pub level: Option<NotificationLevel>,
    pub user_id: Option<String>,
    pub time_window_seconds: u64,
    pub max_batch_size: usize,
    pub enabled: bool,
}

impl AggregationRule {
    /// Check if notification matches this rule
    pub fn matches(&self, notification: &Notification) -> bool {
        if !self.enabled {
            return false;
        }

        if let Some(ref category) = self.category {
            if &notification.category != category {
                return false;
            }
        }

        if let Some(ref level) = self.level {
            if &notification.level != level {
                return false;
            }
        }

        if let Some(ref user_id) = self.user_id {
            if &notification.user_id != user_id {
                return false;
            }
        }

        true
    }
}

/// Aggregated notification batch
#[derive(Debug, Clone)]
pub struct NotificationBatch {
    pub rule_id: String,
    pub notifications: Vec<Notification>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

impl NotificationBatch {
    /// Create aggregate notification from batch
    pub fn create_aggregate(&self) -> Notification {
        if self.notifications.is_empty() {
            panic!("Cannot create aggregate from empty batch");
        }

        let first = &self.notifications[0];
        let count = self.notifications.len();

        let title = if count == 1 {
            first.title.clone()
        } else {
            format!("{} notifications", count)
        };

        let message = if count == 1 {
            first.message.clone()
        } else {
            self.create_batch_summary()
        };

        let mut aggregate = Notification::new(
            first.user_id.clone(),
            self.get_highest_level(),
            title,
            message,
        );

        aggregate.priority = self.get_highest_priority();
        aggregate.category = first.category.clone();
        aggregate.organization_id = first.organization_id.clone();

        // Add metadata about batched notifications
        aggregate.set_metadata(
            "batched_count",
            serde_json::json!(count),
        );
        aggregate.set_metadata(
            "batched_ids",
            serde_json::json!(self.notifications.iter().map(|n| n.id).collect::<Vec<_>>()),
        );

        aggregate
    }

    /// Create summary message
    fn create_batch_summary(&self) -> String {
        let mut summary = String::new();
        let count = self.notifications.len();

        summary.push_str(&format!("You have {} new notifications:\n\n", count));

        for (i, notif) in self.notifications.iter().take(5).enumerate() {
            summary.push_str(&format!("{}. {}\n", i + 1, notif.title));
        }

        if count > 5 {
            summary.push_str(&format!("\n...and {} more", count - 5));
        }

        summary
    }

    /// Get highest priority level
    fn get_highest_priority(&self) -> Priority {
        self.notifications
            .iter()
            .map(|n| n.priority)
            .max()
            .unwrap_or(Priority::Normal)
    }

    /// Get highest notification level
    fn get_highest_level(&self) -> NotificationLevel {
        let max_priority = self
            .notifications
            .iter()
            .map(|n| n.level.priority())
            .max()
            .unwrap_or(1);

        match max_priority {
            5 => NotificationLevel::Alert,
            4 => NotificationLevel::Error,
            3 => NotificationLevel::Warning,
            2 => NotificationLevel::Success,
            _ => NotificationLevel::Info,
        }
    }
}

/// Notification aggregator
pub struct NotificationAggregator {
    rules: Arc<RwLock<HashMap<String, AggregationRule>>>,
    batches: Arc<RwLock<HashMap<String, Vec<NotificationBatch>>>>,
    dispatcher: Arc<NotificationDispatcher>,
    shutdown_tx: Option<tokio::sync::mpsc::Sender<()>>,
}

impl NotificationAggregator {
    /// Create a new aggregator
    pub fn new(dispatcher: Arc<NotificationDispatcher>) -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            batches: Arc::new(RwLock::new(HashMap::new())),
            dispatcher,
            shutdown_tx: None,
        }
    }

    /// Start the aggregator
    pub async fn start(&mut self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let rules = Arc::clone(&self.rules);
        let batches = Arc::clone(&self.batches);
        let dispatcher = Arc::clone(&self.dispatcher);

        // Spawn batch processor task
        tokio::spawn(async move {
            tracing::info!("Notification aggregator started");

            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        Self::process_batches(&rules, &batches, &dispatcher).await;
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Notification aggregator shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Process and send batches
    async fn process_batches(
        rules: &Arc<RwLock<HashMap<String, AggregationRule>>>,
        batches: &Arc<RwLock<HashMap<String, Vec<NotificationBatch>>>>,
        dispatcher: &Arc<NotificationDispatcher>,
    ) {
        let now = Utc::now();
        let mut to_send = Vec::new();

        // Find batches to send
        {
            let rules_lock = rules.read().await;
            let mut batches_lock = batches.write().await;

            for (rule_id, rule) in rules_lock.iter() {
                if !rule.enabled {
                    continue;
                }

                if let Some(user_batches) = batches_lock.get_mut(rule_id) {
                    user_batches.retain(|batch| {
                        let age = now.signed_duration_since(batch.created_at);
                        let should_send = age.num_seconds() as u64 >= rule.time_window_seconds
                            || batch.notifications.len() >= rule.max_batch_size;

                        if should_send {
                            to_send.push(batch.clone());
                            false
                        } else {
                            true
                        }
                    });
                }
            }
        }

        // Send aggregated notifications
        for batch in to_send {
            let aggregate = batch.create_aggregate();

            tracing::info!(
                "Sending aggregated notification with {} items",
                batch.notifications.len()
            );

            if let Err(e) = dispatcher.enqueue(aggregate, vec![]).await {
                tracing::error!("Failed to dispatch aggregated notification: {}", e);
            }
        }
    }

    /// Add a notification to aggregation
    pub async fn add_notification(&self, notification: Notification) -> Result<bool> {
        let rules_lock = self.rules.read().await;

        // Find matching rule
        let matching_rule = rules_lock
            .values()
            .find(|rule| rule.matches(&notification))
            .cloned();

        drop(rules_lock);

        if let Some(rule) = matching_rule {
            let mut batches_lock = self.batches.write().await;

            let user_batches = batches_lock
                .entry(rule.id.clone())
                .or_insert_with(Vec::new);

            // Find or create batch for this user
            let batch = user_batches
                .iter_mut()
                .find(|b| {
                    !b.notifications.is_empty()
                        && b.notifications[0].user_id == notification.user_id
                });

            if let Some(batch) = batch {
                batch.notifications.push(notification);
                batch.last_updated = Utc::now();
            } else {
                user_batches.push(NotificationBatch {
                    rule_id: rule.id.clone(),
                    notifications: vec![notification],
                    created_at: Utc::now(),
                    last_updated: Utc::now(),
                });
            }

            Ok(true) // Notification was aggregated
        } else {
            Ok(false) // No matching rule, should send immediately
        }
    }

    /// Add an aggregation rule
    pub async fn add_rule(&self, rule: AggregationRule) -> Result<()> {
        self.rules.write().await.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Remove an aggregation rule
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        self.rules.write().await.remove(rule_id);
        self.batches.write().await.remove(rule_id);
        Ok(())
    }

    /// Get aggregation rule
    pub async fn get_rule(&self, rule_id: &str) -> Option<AggregationRule> {
        self.rules.read().await.get(rule_id).cloned()
    }

    /// List all rules
    pub async fn list_rules(&self) -> Vec<AggregationRule> {
        self.rules.read().await.values().cloned().collect()
    }

    /// Enable/disable rule
    pub async fn set_rule_enabled(&self, rule_id: &str, enabled: bool) -> Result<()> {
        let mut rules = self.rules.write().await;
        if let Some(rule) = rules.get_mut(rule_id) {
            rule.enabled = enabled;
            Ok(())
        } else {
            Err(NotificationError::Aggregation(format!(
                "Rule not found: {}",
                rule_id
            )))
        }
    }

    /// Flush all batches immediately
    pub async fn flush_all(&self) -> Result<()> {
        let rules = self.rules.read().await;
        let mut batches_lock = self.batches.write().await;

        for (rule_id, user_batches) in batches_lock.iter_mut() {
            if let Some(rule) = rules.get(rule_id) {
                if !rule.enabled {
                    continue;
                }

                for batch in user_batches.drain(..) {
                    let aggregate = batch.create_aggregate();
                    self.dispatcher.enqueue(aggregate, vec![]).await?;
                }
            }
        }

        Ok(())
    }

    /// Flush batches for a specific rule
    pub async fn flush_rule(&self, rule_id: &str) -> Result<()> {
        let mut batches_lock = self.batches.write().await;

        if let Some(user_batches) = batches_lock.get_mut(rule_id) {
            for batch in user_batches.drain(..) {
                let aggregate = batch.create_aggregate();
                self.dispatcher.enqueue(aggregate, vec![]).await?;
            }
        }

        Ok(())
    }

    /// Get statistics
    pub async fn stats(&self) -> AggregatorStats {
        let rules = self.rules.read().await;
        let batches = self.batches.read().await;

        let total_rules = rules.len();
        let enabled_rules = rules.values().filter(|r| r.enabled).count();

        let mut total_batches = 0;
        let mut total_pending = 0;

        for user_batches in batches.values() {
            total_batches += user_batches.len();
            total_pending += user_batches
                .iter()
                .map(|b| b.notifications.len())
                .sum::<usize>();
        }

        AggregatorStats {
            total_rules,
            enabled_rules,
            total_batches,
            total_pending,
        }
    }

    /// Shutdown the aggregator
    pub async fn shutdown(&self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(()).await;
        }
    }

    /// Register default aggregation rules
    pub async fn register_default_rules(&self) -> Result<()> {
        // Aggregate info notifications
        self.add_rule(AggregationRule {
            id: "aggregate_info".to_string(),
            name: "Aggregate Info Notifications".to_string(),
            description: "Batch low-priority info notifications".to_string(),
            category: None,
            level: Some(NotificationLevel::Info),
            user_id: None,
            time_window_seconds: 300, // 5 minutes
            max_batch_size: 10,
            enabled: true,
        })
        .await?;

        // Aggregate success notifications
        self.add_rule(AggregationRule {
            id: "aggregate_success".to_string(),
            name: "Aggregate Success Notifications".to_string(),
            description: "Batch success notifications".to_string(),
            category: None,
            level: Some(NotificationLevel::Success),
            user_id: None,
            time_window_seconds: 180, // 3 minutes
            max_batch_size: 5,
            enabled: true,
        })
        .await?;

        Ok(())
    }
}

/// Aggregator statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatorStats {
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub total_batches: usize,
    pub total_pending: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_matching() {
        let rule = AggregationRule {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test rule".to_string(),
            category: Some(NotificationCategory::System),
            level: Some(NotificationLevel::Info),
            user_id: None,
            time_window_seconds: 300,
            max_batch_size: 10,
            enabled: true,
        };

        let mut notification = Notification::new("user1", NotificationLevel::Info, "Test", "Msg");
        notification.category = NotificationCategory::System;

        assert!(rule.matches(&notification));

        notification.level = NotificationLevel::Error;
        assert!(!rule.matches(&notification));
    }
}
