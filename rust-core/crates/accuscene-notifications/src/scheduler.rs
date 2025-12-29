//! Scheduled notifications with cron support

use crate::dispatcher::NotificationDispatcher;
use crate::error::{NotificationError, Result};
use crate::types::Notification;
use chrono::{DateTime, Utc};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Scheduled notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledNotification {
    pub id: Uuid,
    pub notification: Notification,
    pub schedule: String, // Cron expression
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub channels: Vec<String>,
}

impl ScheduledNotification {
    /// Create a new scheduled notification
    pub fn new(notification: Notification, schedule: String, channels: Vec<String>) -> Result<Self> {
        // Validate cron expression
        Schedule::from_str(&schedule)
            .map_err(|e| NotificationError::Scheduling(format!("Invalid cron expression: {}", e)))?;

        let schedule_parsed = Schedule::from_str(&schedule).unwrap();
        let next_run = schedule_parsed.upcoming(Utc).next();

        Ok(Self {
            id: Uuid::new_v4(),
            notification,
            schedule,
            enabled: true,
            last_run: None,
            next_run,
            created_at: Utc::now(),
            channels,
        })
    }

    /// Update next run time
    pub fn update_next_run(&mut self) -> Result<()> {
        let schedule = Schedule::from_str(&self.schedule)
            .map_err(|e| NotificationError::Scheduling(format!("Invalid cron expression: {}", e)))?;

        self.next_run = schedule.upcoming(Utc).next();
        Ok(())
    }

    /// Check if should run now
    pub fn should_run_now(&self) -> bool {
        if !self.enabled {
            return false;
        }

        if let Some(next_run) = self.next_run {
            Utc::now() >= next_run
        } else {
            false
        }
    }
}

/// Notification scheduler
pub struct NotificationScheduler {
    scheduled: Arc<RwLock<HashMap<Uuid, ScheduledNotification>>>,
    dispatcher: Arc<NotificationDispatcher>,
    shutdown_tx: Option<tokio::sync::mpsc::Sender<()>>,
}

impl NotificationScheduler {
    /// Create a new scheduler
    pub fn new(dispatcher: Arc<NotificationDispatcher>) -> Self {
        Self {
            scheduled: Arc::new(RwLock::new(HashMap::new())),
            dispatcher,
            shutdown_tx: None,
        }
    }

    /// Start the scheduler
    pub async fn start(&mut self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let scheduled = Arc::clone(&self.scheduled);
        let dispatcher = Arc::clone(&self.dispatcher);

        // Spawn scheduler task
        tokio::spawn(async move {
            tracing::info!("Notification scheduler started");

            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        Self::process_scheduled(&scheduled, &dispatcher).await;
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Notification scheduler shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Process scheduled notifications
    async fn process_scheduled(
        scheduled: &Arc<RwLock<HashMap<Uuid, ScheduledNotification>>>,
        dispatcher: &Arc<NotificationDispatcher>,
    ) {
        let now = Utc::now();
        let mut to_run = Vec::new();

        // Find notifications to run
        {
            let scheduled_lock = scheduled.read().await;
            for (id, sched_notif) in scheduled_lock.iter() {
                if sched_notif.should_run_now() {
                    to_run.push((*id, sched_notif.clone()));
                }
            }
        }

        // Run notifications
        for (id, mut sched_notif) in to_run {
            tracing::info!("Running scheduled notification: {}", id);

            // Dispatch notification
            match dispatcher
                .enqueue(sched_notif.notification.clone(), sched_notif.channels.clone())
                .await
            {
                Ok(_) => {
                    tracing::info!("Scheduled notification {} dispatched", id);

                    // Update last run and next run
                    sched_notif.last_run = Some(now);
                    if let Err(e) = sched_notif.update_next_run() {
                        tracing::error!("Failed to update next run for {}: {}", id, e);
                    }

                    // Update in storage
                    scheduled.write().await.insert(id, sched_notif);
                }
                Err(e) => {
                    tracing::error!("Failed to dispatch scheduled notification {}: {}", id, e);
                }
            }
        }
    }

    /// Schedule a notification
    pub async fn schedule(
        &self,
        notification: Notification,
        cron_expression: String,
        channels: Vec<String>,
    ) -> Result<Uuid> {
        let scheduled_notif =
            ScheduledNotification::new(notification, cron_expression, channels)?;
        let id = scheduled_notif.id;

        self.scheduled
            .write()
            .await
            .insert(id, scheduled_notif);

        tracing::info!("Scheduled notification: {}", id);
        Ok(id)
    }

    /// Schedule a one-time notification
    pub async fn schedule_once(
        &self,
        notification: Notification,
        scheduled_time: DateTime<Utc>,
        channels: Vec<String>,
    ) -> Result<Uuid> {
        // Create a cron expression for the specific time
        let cron_expr = format!(
            "{} {} {} {} * {}",
            scheduled_time.minute(),
            scheduled_time.hour(),
            scheduled_time.day(),
            scheduled_time.month(),
            scheduled_time.year()
        );

        let mut scheduled_notif =
            ScheduledNotification::new(notification, cron_expr, channels)?;
        scheduled_notif.next_run = Some(scheduled_time);

        let id = scheduled_notif.id;
        self.scheduled
            .write()
            .await
            .insert(id, scheduled_notif);

        tracing::info!("Scheduled one-time notification: {} at {}", id, scheduled_time);
        Ok(id)
    }

    /// Cancel a scheduled notification
    pub async fn cancel(&self, id: Uuid) -> Result<()> {
        if self.scheduled.write().await.remove(&id).is_some() {
            tracing::info!("Cancelled scheduled notification: {}", id);
            Ok(())
        } else {
            Err(NotificationError::NotificationNotFound(id.to_string()))
        }
    }

    /// Enable/disable a scheduled notification
    pub async fn set_enabled(&self, id: Uuid, enabled: bool) -> Result<()> {
        let mut scheduled = self.scheduled.write().await;
        if let Some(sched_notif) = scheduled.get_mut(&id) {
            sched_notif.enabled = enabled;
            tracing::info!(
                "Scheduled notification {} {}",
                id,
                if enabled { "enabled" } else { "disabled" }
            );
            Ok(())
        } else {
            Err(NotificationError::NotificationNotFound(id.to_string()))
        }
    }

    /// Get a scheduled notification
    pub async fn get(&self, id: Uuid) -> Option<ScheduledNotification> {
        self.scheduled.read().await.get(&id).cloned()
    }

    /// List all scheduled notifications
    pub async fn list(&self) -> Vec<ScheduledNotification> {
        self.scheduled.read().await.values().cloned().collect()
    }

    /// List scheduled notifications for a user
    pub async fn list_for_user(&self, user_id: &str) -> Vec<ScheduledNotification> {
        self.scheduled
            .read()
            .await
            .values()
            .filter(|s| s.notification.user_id == user_id)
            .cloned()
            .collect()
    }

    /// Update schedule
    pub async fn update_schedule(&self, id: Uuid, cron_expression: String) -> Result<()> {
        // Validate cron expression
        Schedule::from_str(&cron_expression).map_err(|e| {
            NotificationError::Scheduling(format!("Invalid cron expression: {}", e))
        })?;

        let mut scheduled = self.scheduled.write().await;
        if let Some(sched_notif) = scheduled.get_mut(&id) {
            sched_notif.schedule = cron_expression;
            sched_notif.update_next_run()?;
            tracing::info!("Updated schedule for notification: {}", id);
            Ok(())
        } else {
            Err(NotificationError::NotificationNotFound(id.to_string()))
        }
    }

    /// Shutdown the scheduler
    pub async fn shutdown(&self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(()).await;
        }
    }

    /// Get scheduler statistics
    pub async fn stats(&self) -> SchedulerStats {
        let scheduled = self.scheduled.read().await;
        let total = scheduled.len();
        let enabled = scheduled.values().filter(|s| s.enabled).count();
        let disabled = total - enabled;

        SchedulerStats {
            total,
            enabled,
            disabled,
        }
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
}

/// Common cron expressions
pub mod cron_expressions {
    /// Every minute
    pub const EVERY_MINUTE: &str = "* * * * *";

    /// Every 5 minutes
    pub const EVERY_5_MINUTES: &str = "*/5 * * * *";

    /// Every hour
    pub const EVERY_HOUR: &str = "0 * * * *";

    /// Every day at midnight
    pub const DAILY_MIDNIGHT: &str = "0 0 * * *";

    /// Every day at 9 AM
    pub const DAILY_9AM: &str = "0 9 * * *";

    /// Every Monday at 9 AM
    pub const WEEKLY_MONDAY_9AM: &str = "0 9 * * MON";

    /// First day of month at midnight
    pub const MONTHLY_FIRST: &str = "0 0 1 * *";

    /// Every weekday at 9 AM
    pub const WEEKDAYS_9AM: &str = "0 9 * * MON-FRI";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NotificationLevel, Priority};

    #[test]
    fn test_cron_validation() {
        let notification = Notification::new("user1", NotificationLevel::Info, "Test", "Message");

        // Valid cron
        let result =
            ScheduledNotification::new(notification.clone(), "0 9 * * *".to_string(), vec![]);
        assert!(result.is_ok());

        // Invalid cron
        let result =
            ScheduledNotification::new(notification.clone(), "invalid".to_string(), vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_should_run_now() {
        let notification = Notification::new("user1", NotificationLevel::Info, "Test", "Message");
        let mut scheduled =
            ScheduledNotification::new(notification, "0 9 * * *".to_string(), vec![]).unwrap();

        // Set next_run to past
        scheduled.next_run = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(scheduled.should_run_now());

        // Set next_run to future
        scheduled.next_run = Some(Utc::now() + chrono::Duration::hours(1));
        assert!(!scheduled.should_run_now());

        // Disabled
        scheduled.enabled = false;
        scheduled.next_run = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(!scheduled.should_run_now());
    }
}
