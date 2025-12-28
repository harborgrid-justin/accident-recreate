//! Notification persistence and history management

use crate::error::{NotificationError, Result};
use crate::types::{Notification, NotificationStats};
use chrono::{DateTime, Duration, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Notification store for persistence
pub struct NotificationStore {
    pool: PgPool,
}

impl NotificationStore {
    /// Create a new notification store
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize database schema
    pub async fn initialize(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notifications (
                id UUID PRIMARY KEY,
                user_id VARCHAR(255) NOT NULL,
                organization_id VARCHAR(255),
                level VARCHAR(50) NOT NULL,
                priority INTEGER NOT NULL,
                category VARCHAR(100) NOT NULL,
                title TEXT NOT NULL,
                message TEXT NOT NULL,
                html_message TEXT,
                actions JSONB,
                metadata JSONB,
                related_entity_id VARCHAR(255),
                related_entity_type VARCHAR(100),
                read BOOLEAN DEFAULT false,
                read_at TIMESTAMP WITH TIME ZONE,
                archived BOOLEAN DEFAULT false,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                expires_at TIMESTAMP WITH TIME ZONE,
                sender JSONB,
                template_id VARCHAR(255),
                template_vars JSONB
            );

            CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notifications(user_id);
            CREATE INDEX IF NOT EXISTS idx_notifications_org_id ON notifications(organization_id);
            CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(read);
            CREATE INDEX IF NOT EXISTS idx_notifications_level ON notifications(level);
            CREATE INDEX IF NOT EXISTS idx_notifications_category ON notifications(category);
            CREATE INDEX IF NOT EXISTS idx_notifications_expires_at ON notifications(expires_at);

            CREATE TABLE IF NOT EXISTS notification_delivery_status (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
                channel VARCHAR(50) NOT NULL,
                status VARCHAR(50) NOT NULL,
                attempts INTEGER NOT NULL DEFAULT 0,
                last_attempt_at TIMESTAMP WITH TIME ZONE,
                delivered_at TIMESTAMP WITH TIME ZONE,
                error_message TEXT,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_delivery_status_notification_id ON notification_delivery_status(notification_id);
            CREATE INDEX IF NOT EXISTS idx_delivery_status_channel ON notification_delivery_status(channel);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Save a notification
    pub async fn save(&self, notification: &Notification) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO notifications (
                id, user_id, organization_id, level, priority, category,
                title, message, html_message, actions, metadata,
                related_entity_id, related_entity_type, read, read_at,
                archived, created_at, expires_at, sender, template_id, template_vars
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            ON CONFLICT (id) DO UPDATE SET
                read = EXCLUDED.read,
                read_at = EXCLUDED.read_at,
                archived = EXCLUDED.archived
            "#,
        )
        .bind(notification.id)
        .bind(&notification.user_id)
        .bind(&notification.organization_id)
        .bind(format!("{:?}", notification.level))
        .bind(notification.priority as i32)
        .bind(serde_json::to_value(&notification.category)?)
        .bind(&notification.title)
        .bind(&notification.message)
        .bind(&notification.html_message)
        .bind(serde_json::to_value(&notification.actions)?)
        .bind(serde_json::to_value(&notification.metadata)?)
        .bind(&notification.related_entity_id)
        .bind(&notification.related_entity_type)
        .bind(notification.read)
        .bind(notification.read_at)
        .bind(notification.archived)
        .bind(notification.created_at)
        .bind(notification.expires_at)
        .bind(serde_json::to_value(&notification.sender)?)
        .bind(&notification.template_id)
        .bind(serde_json::to_value(&notification.template_vars)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a notification by ID
    pub async fn get(&self, id: Uuid) -> Result<Option<Notification>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM notifications WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_notification(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get notifications for a user
    pub async fn get_for_user(
        &self,
        user_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Notification>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM notifications
            WHERE user_id = $1 AND archived = false AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| self.row_to_notification(row))
            .collect()
    }

    /// Get unread notifications for a user
    pub async fn get_unread(
        &self,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<Notification>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM notifications
            WHERE user_id = $1 AND read = false AND archived = false AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| self.row_to_notification(row))
            .collect()
    }

    /// Mark notification as read
    pub async fn mark_read(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET read = true, read_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark all notifications as read for a user
    pub async fn mark_all_read(&self, user_id: &str) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET read = true, read_at = NOW()
            WHERE user_id = $1 AND read = false
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Mark notification as unread
    pub async fn mark_unread(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET read = false, read_at = NULL
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Archive notification
    pub async fn archive(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET archived = true
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete notification
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM notifications WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete expired notifications
    pub async fn delete_expired(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM notifications
            WHERE expires_at IS NOT NULL AND expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Delete old archived notifications
    pub async fn delete_old_archived(&self, days: i64) -> Result<u64> {
        let cutoff_date = Utc::now() - Duration::days(days);

        let result = sqlx::query(
            r#"
            DELETE FROM notifications
            WHERE archived = true AND created_at < $1
            "#,
        )
        .bind(cutoff_date)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get notification count for a user
    pub async fn count_for_user(&self, user_id: &str) -> Result<u64> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM notifications
            WHERE user_id = $1 AND archived = false AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get::<i64, _>("count") as u64)
    }

    /// Get unread count for a user
    pub async fn count_unread(&self, user_id: &str) -> Result<u64> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM notifications
            WHERE user_id = $1 AND read = false AND archived = false AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get::<i64, _>("count") as u64)
    }

    /// Get notification statistics for a user
    pub async fn get_stats(&self, user_id: &str) -> Result<NotificationStats> {
        let total = self.count_for_user(user_id).await?;
        let unread = self.count_unread(user_id).await?;

        // Get counts by level
        let level_rows = sqlx::query(
            r#"
            SELECT level, COUNT(*) as count FROM notifications
            WHERE user_id = $1 AND archived = false AND (expires_at IS NULL OR expires_at > NOW())
            GROUP BY level
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut by_level = std::collections::HashMap::new();
        for row in level_rows {
            by_level.insert(
                row.get::<String, _>("level"),
                row.get::<i64, _>("count") as u64,
            );
        }

        // Get counts by category
        let category_rows = sqlx::query(
            r#"
            SELECT category, COUNT(*) as count FROM notifications
            WHERE user_id = $1 AND archived = false AND (expires_at IS NULL OR expires_at > NOW())
            GROUP BY category
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut by_category = std::collections::HashMap::new();
        for row in category_rows {
            by_category.insert(
                row.get::<String, _>("category"),
                row.get::<i64, _>("count") as u64,
            );
        }

        Ok(NotificationStats {
            total,
            unread,
            by_level,
            by_category,
            by_channel: std::collections::HashMap::new(), // Populated from delivery status
        })
    }

    /// Convert database row to Notification
    fn row_to_notification(&self, row: sqlx::postgres::PgRow) -> Result<Notification> {
        use sqlx::Row;

        Ok(Notification {
            id: row.get("id"),
            user_id: row.get("user_id"),
            organization_id: row.get("organization_id"),
            level: serde_json::from_str(&format!("\"{}\"", row.get::<String, _>("level")))?,
            priority: match row.get::<i32, _>("priority") {
                1 => crate::types::Priority::Low,
                2 => crate::types::Priority::Normal,
                3 => crate::types::Priority::High,
                4 => crate::types::Priority::Urgent,
                5 => crate::types::Priority::Critical,
                _ => crate::types::Priority::Normal,
            },
            category: serde_json::from_value(row.get("category"))?,
            title: row.get("title"),
            message: row.get("message"),
            html_message: row.get("html_message"),
            actions: serde_json::from_value(row.get("actions"))?,
            metadata: serde_json::from_value(row.get("metadata"))?,
            related_entity_id: row.get("related_entity_id"),
            related_entity_type: row.get("related_entity_type"),
            read: row.get("read"),
            read_at: row.get("read_at"),
            archived: row.get("archived"),
            created_at: row.get("created_at"),
            expires_at: row.get("expires_at"),
            sender: serde_json::from_value(row.get("sender"))?,
            template_id: row.get("template_id"),
            template_vars: serde_json::from_value(row.get("template_vars"))?,
        })
    }
}
