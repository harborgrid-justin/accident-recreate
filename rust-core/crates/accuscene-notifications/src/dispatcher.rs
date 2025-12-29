//! Multi-channel notification dispatcher with priority queue

use crate::channel::{Channel, ChannelRegistry};
use crate::config::DispatcherConfig;
use crate::error::{NotificationError, Result};
use crate::preferences::PreferenceManager;
use crate::types::{DeliveryState, DeliveryStatus, Notification, Priority};
use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Notification with dispatch metadata
#[derive(Clone)]
struct DispatchItem {
    notification: Notification,
    channels: Vec<String>,
    retry_count: u32,
}

impl DispatchItem {
    fn priority_score(&self) -> u32 {
        // Higher score = higher priority
        let base_score = match self.notification.priority {
            Priority::Critical => 10000,
            Priority::Urgent => 8000,
            Priority::High => 6000,
            Priority::Normal => 4000,
            Priority::Low => 2000,
        };

        let level_bonus = self.notification.level.priority() as u32 * 100;
        let retry_penalty = self.retry_count * 10;

        base_score + level_bonus - retry_penalty
    }
}

/// Multi-channel notification dispatcher
pub struct NotificationDispatcher {
    config: DispatcherConfig,
    channel_registry: Arc<ChannelRegistry>,
    preference_manager: Arc<PreferenceManager>,
    queue: Arc<RwLock<PriorityQueue<Uuid, Reverse<u32>>>>,
    pending_items: Arc<RwLock<HashMap<Uuid, DispatchItem>>>,
    delivery_statuses: Arc<RwLock<HashMap<Uuid, Vec<DeliveryStatus>>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl NotificationDispatcher {
    /// Create a new dispatcher
    pub fn new(
        config: DispatcherConfig,
        channel_registry: Arc<ChannelRegistry>,
        preference_manager: Arc<PreferenceManager>,
    ) -> Self {
        Self {
            config,
            channel_registry,
            preference_manager,
            queue: Arc::new(RwLock::new(PriorityQueue::new())),
            pending_items: Arc::new(RwLock::new(HashMap::new())),
            delivery_statuses: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: None,
        }
    }

    /// Start the dispatcher
    pub async fn start(&mut self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let worker_count = self.config.worker_count;
        let queue = Arc::clone(&self.queue);
        let pending_items = Arc::clone(&self.pending_items);
        let delivery_statuses = Arc::clone(&self.delivery_statuses);
        let channel_registry = Arc::clone(&self.channel_registry);
        let batch_size = self.config.batch_size;
        let batch_timeout = std::time::Duration::from_millis(self.config.batch_timeout_ms);

        // Spawn worker tasks
        for worker_id in 0..worker_count {
            let queue = Arc::clone(&queue);
            let pending_items = Arc::clone(&pending_items);
            let delivery_statuses = Arc::clone(&delivery_statuses);
            let channel_registry = Arc::clone(&channel_registry);

            tokio::spawn(async move {
                tracing::info!("Dispatcher worker {} started", worker_id);

                loop {
                    tokio::time::sleep(batch_timeout).await;

                    // Get batch of items from queue
                    let mut batch = Vec::new();
                    {
                        let mut queue_lock = queue.write().await;
                        for _ in 0..batch_size {
                            if let Some((notification_id, _)) = queue_lock.pop() {
                                batch.push(notification_id);
                            } else {
                                break;
                            }
                        }
                    }

                    if batch.is_empty() {
                        continue;
                    }

                    // Process batch
                    for notification_id in batch {
                        let item = {
                            let items = pending_items.read().await;
                            items.get(&notification_id).cloned()
                        };

                        if let Some(item) = item {
                            Self::dispatch_notification(
                                item,
                                &channel_registry,
                                &delivery_statuses,
                            )
                            .await;

                            // Remove from pending
                            pending_items.write().await.remove(&notification_id);
                        }
                    }
                }
            });
        }

        // Wait for shutdown signal
        tokio::spawn(async move {
            shutdown_rx.recv().await;
            tracing::info!("Dispatcher shutting down");
        });

        Ok(())
    }

    /// Dispatch a notification
    async fn dispatch_notification(
        item: DispatchItem,
        channel_registry: &Arc<ChannelRegistry>,
        delivery_statuses: &Arc<RwLock<HashMap<Uuid, Vec<DeliveryStatus>>>>,
    ) {
        let notification = &item.notification;
        let mut statuses = Vec::new();

        // Get channels to use
        let channels_to_use: Vec<_> = if item.channels.is_empty() {
            channel_registry.get_channels_for(notification)
        } else {
            item.channels
                .iter()
                .filter_map(|name| channel_registry.get_channel(name))
                .collect()
        };

        // Deliver to each channel
        for channel in channels_to_use {
            match channel.deliver(notification).await {
                Ok(status) => {
                    tracing::info!(
                        "Notification {} delivered via {}: {:?}",
                        notification.id,
                        channel.name(),
                        status.status
                    );
                    statuses.push(status);
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to deliver notification {} via {}: {}",
                        notification.id,
                        channel.name(),
                        e
                    );
                    statuses.push(DeliveryStatus {
                        notification_id: notification.id,
                        channel: channel.name().to_string(),
                        status: DeliveryState::Failed,
                        attempts: item.retry_count + 1,
                        last_attempt_at: Some(chrono::Utc::now()),
                        delivered_at: None,
                        error_message: Some(e.to_string()),
                    });
                }
            }
        }

        // Store delivery statuses
        delivery_statuses
            .write()
            .await
            .insert(notification.id, statuses);
    }

    /// Enqueue a notification for dispatch
    pub async fn enqueue(
        &self,
        notification: Notification,
        channels: Vec<String>,
    ) -> Result<Uuid> {
        // Check queue capacity
        {
            let queue = self.queue.read().await;
            if queue.len() >= self.config.queue_capacity {
                return Err(NotificationError::QueueFull);
            }
        }

        // Apply user preferences
        let channels = if channels.is_empty() {
            self.preference_manager
                .get_enabled_channels(&notification.user_id)
                .await
                .unwrap_or_else(|_| vec!["in_app".to_string()])
        } else {
            channels
        };

        let item = DispatchItem {
            notification: notification.clone(),
            channels,
            retry_count: 0,
        };

        let notification_id = notification.id;
        let priority = Reverse(item.priority_score());

        // Add to pending items
        self.pending_items
            .write()
            .await
            .insert(notification_id, item);

        // Add to priority queue
        self.queue.write().await.push(notification_id, priority);

        tracing::debug!(
            "Enqueued notification {} with priority {:?}",
            notification_id,
            notification.priority
        );

        Ok(notification_id)
    }

    /// Enqueue multiple notifications
    pub async fn enqueue_batch(
        &self,
        notifications: Vec<Notification>,
        channels: Vec<String>,
    ) -> Result<Vec<Uuid>> {
        let mut ids = Vec::new();

        for notification in notifications {
            match self.enqueue(notification, channels.clone()).await {
                Ok(id) => ids.push(id),
                Err(e) => {
                    tracing::error!("Failed to enqueue notification: {}", e);
                }
            }
        }

        Ok(ids)
    }

    /// Get delivery status for a notification
    pub async fn get_delivery_status(&self, notification_id: Uuid) -> Option<Vec<DeliveryStatus>> {
        self.delivery_statuses
            .read()
            .await
            .get(&notification_id)
            .cloned()
    }

    /// Get queue size
    pub async fn queue_size(&self) -> usize {
        self.queue.read().await.len()
    }

    /// Get pending items count
    pub async fn pending_count(&self) -> usize {
        self.pending_items.read().await.len()
    }

    /// Retry failed deliveries
    pub async fn retry_failed(&self, notification_id: Uuid) -> Result<()> {
        let statuses = self
            .delivery_statuses
            .read()
            .await
            .get(&notification_id)
            .cloned();

        if let Some(statuses) = statuses {
            let failed_channels: Vec<String> = statuses
                .iter()
                .filter(|s| s.status == DeliveryState::Failed)
                .map(|s| s.channel.clone())
                .collect();

            if !failed_channels.is_empty() {
                let item = self
                    .pending_items
                    .read()
                    .await
                    .get(&notification_id)
                    .cloned();

                if let Some(mut item) = item {
                    item.retry_count += 1;
                    item.channels = failed_channels;

                    let priority = Reverse(item.priority_score());
                    self.pending_items
                        .write()
                        .await
                        .insert(notification_id, item);
                    self.queue
                        .write()
                        .await
                        .push(notification_id, priority);
                }
            }
        }

        Ok(())
    }

    /// Shutdown the dispatcher
    pub async fn shutdown(&self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(()).await;
        }
    }

    /// Get statistics
    pub async fn stats(&self) -> DispatcherStats {
        let queue_size = self.queue_size().await;
        let pending_count = self.pending_count().await;

        let delivery_statuses = self.delivery_statuses.read().await;
        let total_delivered = delivery_statuses
            .values()
            .flatten()
            .filter(|s| s.status == DeliveryState::Delivered)
            .count();
        let total_failed = delivery_statuses
            .values()
            .flatten()
            .filter(|s| s.status == DeliveryState::Failed)
            .count();

        DispatcherStats {
            queue_size,
            pending_count,
            total_delivered,
            total_failed,
        }
    }
}

/// Dispatcher statistics
#[derive(Debug, Clone)]
pub struct DispatcherStats {
    pub queue_size: usize,
    pub pending_count: usize,
    pub total_delivered: usize,
    pub total_failed: usize,
}
