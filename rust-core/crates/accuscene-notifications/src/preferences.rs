//! User notification preferences management

use crate::error::{NotificationError, Result};
use crate::types::{NotificationCategory, NotificationLevel};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

/// User notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: String,
    pub enabled_channels: Vec<String>,
    pub quiet_hours: Option<QuietHours>,
    pub category_preferences: HashMap<String, CategoryPreference>,
    pub level_preferences: HashMap<String, bool>,
    pub digest_enabled: bool,
    pub digest_frequency: DigestFrequency,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            user_id: String::new(),
            enabled_channels: vec!["in_app".to_string()],
            quiet_hours: None,
            category_preferences: HashMap::new(),
            level_preferences: HashMap::new(),
            digest_enabled: false,
            digest_frequency: DigestFrequency::Daily,
        }
    }
}

/// Quiet hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    pub enabled: bool,
    pub start_hour: u8,  // 0-23
    pub end_hour: u8,    // 0-23
    pub timezone: String,
    pub days: Vec<u8>,   // 0-6 (Sunday-Saturday)
}

/// Category-specific preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryPreference {
    pub enabled: bool,
    pub channels: Vec<String>,
    pub min_priority: u8, // 1-5
}

/// Notification digest frequency
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DigestFrequency {
    Hourly,
    Daily,
    Weekly,
}

/// Preference manager
pub struct PreferenceManager {
    pool: PgPool,
}

impl PreferenceManager {
    /// Create a new preference manager
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize database schema
    pub async fn initialize(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notification_preferences (
                user_id VARCHAR(255) PRIMARY KEY,
                enabled_channels JSONB NOT NULL DEFAULT '["in_app"]',
                quiet_hours JSONB,
                category_preferences JSONB NOT NULL DEFAULT '{}',
                level_preferences JSONB NOT NULL DEFAULT '{}',
                digest_enabled BOOLEAN DEFAULT false,
                digest_frequency VARCHAR(50) DEFAULT 'daily',
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_preferences_user_id ON notification_preferences(user_id);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get preferences for a user
    pub async fn get(&self, user_id: &str) -> Result<NotificationPreferences> {
        let row = sqlx::query(
            r#"
            SELECT * FROM notification_preferences WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            use sqlx::Row;

            Ok(NotificationPreferences {
                user_id: row.get("user_id"),
                enabled_channels: serde_json::from_value(row.get("enabled_channels"))?,
                quiet_hours: serde_json::from_value(row.get("quiet_hours"))?,
                category_preferences: serde_json::from_value(row.get("category_preferences"))?,
                level_preferences: serde_json::from_value(row.get("level_preferences"))?,
                digest_enabled: row.get("digest_enabled"),
                digest_frequency: serde_json::from_str(&format!(
                    "\"{}\"",
                    row.get::<String, _>("digest_frequency")
                ))?,
            })
        } else {
            // Return defaults if no preferences exist
            Ok(NotificationPreferences {
                user_id: user_id.to_string(),
                ..Default::default()
            })
        }
    }

    /// Save preferences for a user
    pub async fn save(&self, preferences: &NotificationPreferences) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO notification_preferences (
                user_id, enabled_channels, quiet_hours, category_preferences,
                level_preferences, digest_enabled, digest_frequency, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            ON CONFLICT (user_id) DO UPDATE SET
                enabled_channels = EXCLUDED.enabled_channels,
                quiet_hours = EXCLUDED.quiet_hours,
                category_preferences = EXCLUDED.category_preferences,
                level_preferences = EXCLUDED.level_preferences,
                digest_enabled = EXCLUDED.digest_enabled,
                digest_frequency = EXCLUDED.digest_frequency,
                updated_at = NOW()
            "#,
        )
        .bind(&preferences.user_id)
        .bind(serde_json::to_value(&preferences.enabled_channels)?)
        .bind(serde_json::to_value(&preferences.quiet_hours)?)
        .bind(serde_json::to_value(&preferences.category_preferences)?)
        .bind(serde_json::to_value(&preferences.level_preferences)?)
        .bind(preferences.digest_enabled)
        .bind(format!("{:?}", preferences.digest_frequency).to_lowercase())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update enabled channels
    pub async fn set_enabled_channels(
        &self,
        user_id: &str,
        channels: Vec<String>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO notification_preferences (user_id, enabled_channels, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (user_id) DO UPDATE SET
                enabled_channels = EXCLUDED.enabled_channels,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(serde_json::to_value(channels)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get enabled channels for a user
    pub async fn get_enabled_channels(&self, user_id: &str) -> Result<Vec<String>> {
        let prefs = self.get(user_id).await?;
        Ok(prefs.enabled_channels)
    }

    /// Set quiet hours
    pub async fn set_quiet_hours(&self, user_id: &str, quiet_hours: QuietHours) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO notification_preferences (user_id, quiet_hours, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (user_id) DO UPDATE SET
                quiet_hours = EXCLUDED.quiet_hours,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(serde_json::to_value(quiet_hours)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if user is in quiet hours
    pub async fn is_quiet_hours(&self, user_id: &str) -> Result<bool> {
        let prefs = self.get(user_id).await?;

        if let Some(quiet_hours) = prefs.quiet_hours {
            if !quiet_hours.enabled {
                return Ok(false);
            }

            let now = chrono::Utc::now();
            let hour = now.hour() as u8;
            let weekday = now.weekday().number_from_sunday() as u8 - 1;

            // Check if current day is in quiet hours days
            if !quiet_hours.days.is_empty() && !quiet_hours.days.contains(&weekday) {
                return Ok(false);
            }

            // Check if current hour is in quiet hours range
            if quiet_hours.start_hour <= quiet_hours.end_hour {
                Ok(hour >= quiet_hours.start_hour && hour < quiet_hours.end_hour)
            } else {
                // Handle overnight quiet hours (e.g., 22:00 - 06:00)
                Ok(hour >= quiet_hours.start_hour || hour < quiet_hours.end_hour)
            }
        } else {
            Ok(false)
        }
    }

    /// Set category preference
    pub async fn set_category_preference(
        &self,
        user_id: &str,
        category: NotificationCategory,
        preference: CategoryPreference,
    ) -> Result<()> {
        let mut prefs = self.get(user_id).await?;
        let category_key = match category {
            NotificationCategory::Custom(ref name) => name.clone(),
            _ => format!("{:?}", category).to_lowercase(),
        };
        prefs.category_preferences.insert(category_key, preference);
        self.save(&prefs).await
    }

    /// Get category preference
    pub async fn get_category_preference(
        &self,
        user_id: &str,
        category: &NotificationCategory,
    ) -> Result<Option<CategoryPreference>> {
        let prefs = self.get(user_id).await?;
        let category_key = match category {
            NotificationCategory::Custom(ref name) => name.clone(),
            _ => format!("{:?}", category).to_lowercase(),
        };
        Ok(prefs.category_preferences.get(&category_key).cloned())
    }

    /// Enable/disable digest
    pub async fn set_digest(
        &self,
        user_id: &str,
        enabled: bool,
        frequency: DigestFrequency,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO notification_preferences (user_id, digest_enabled, digest_frequency, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (user_id) DO UPDATE SET
                digest_enabled = EXCLUDED.digest_enabled,
                digest_frequency = EXCLUDED.digest_frequency,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(enabled)
        .bind(format!("{:?}", frequency).to_lowercase())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Set level preference
    pub async fn set_level_preference(
        &self,
        user_id: &str,
        level: NotificationLevel,
        enabled: bool,
    ) -> Result<()> {
        let mut prefs = self.get(user_id).await?;
        prefs
            .level_preferences
            .insert(format!("{:?}", level).to_lowercase(), enabled);
        self.save(&prefs).await
    }

    /// Check if notification should be sent based on preferences
    pub async fn should_send(
        &self,
        user_id: &str,
        level: &NotificationLevel,
        category: &NotificationCategory,
    ) -> Result<bool> {
        let prefs = self.get(user_id).await?;

        // Check if in quiet hours
        if self.is_quiet_hours(user_id).await? {
            // Only send critical/urgent during quiet hours
            if !matches!(level, NotificationLevel::Alert | NotificationLevel::Error) {
                return Ok(false);
            }
        }

        // Check level preference
        let level_key = format!("{:?}", level).to_lowercase();
        if let Some(&enabled) = prefs.level_preferences.get(&level_key) {
            if !enabled {
                return Ok(false);
            }
        }

        // Check category preference
        let category_key = match category {
            NotificationCategory::Custom(ref name) => name.clone(),
            _ => format!("{:?}", category).to_lowercase(),
        };

        if let Some(cat_pref) = prefs.category_preferences.get(&category_key) {
            if !cat_pref.enabled {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Delete preferences for a user
    pub async fn delete(&self, user_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM notification_preferences WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
