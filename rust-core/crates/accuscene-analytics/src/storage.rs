//! Analytics data storage layer

use crate::error::{AnalyticsError, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// Storage backend for analytics data
pub struct AnalyticsStorage {
    data: Arc<DashMap<String, StorageEntry>>,
    retention_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEntry {
    pub key: String,
    pub data: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub metadata: StorageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    pub content_type: String,
    pub size: usize,
    pub tags: Vec<String>,
}

impl AnalyticsStorage {
    pub fn new(retention_seconds: i64) -> Self {
        Self {
            data: Arc::new(DashMap::new()),
            retention_seconds,
        }
    }

    /// Store data with a key
    pub fn put(&self, key: impl Into<String>, data: Vec<u8>, metadata: StorageMetadata) -> Result<()> {
        let key = key.into();
        let entry = StorageEntry {
            key: key.clone(),
            data,
            timestamp: Utc::now(),
            metadata,
        };

        self.data.insert(key, entry);
        self.cleanup();

        Ok(())
    }

    /// Store serializable data
    pub fn put_json<T: Serialize>(&self, key: impl Into<String>, value: &T) -> Result<()> {
        let data = serde_json::to_vec(value)?;
        let metadata = StorageMetadata {
            content_type: "application/json".to_string(),
            size: data.len(),
            tags: vec![],
        };

        self.put(key, data, metadata)
    }

    /// Retrieve data by key
    pub fn get(&self, key: &str) -> Result<Vec<u8>> {
        self.cleanup();

        self.data
            .get(key)
            .map(|entry| entry.data.clone())
            .ok_or_else(|| AnalyticsError::Storage(format!("Key not found: {}", key)))
    }

    /// Retrieve and deserialize JSON data
    pub fn get_json<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T> {
        let data = self.get(key)?;
        serde_json::from_slice(&data).map_err(Into::into)
    }

    /// Delete data by key
    pub fn delete(&self, key: &str) -> Result<()> {
        self.data
            .remove(key)
            .ok_or_else(|| AnalyticsError::Storage(format!("Key not found: {}", key)))?;

        Ok(())
    }

    /// Check if a key exists
    pub fn exists(&self, key: &str) -> bool {
        self.cleanup();
        self.data.contains_key(key)
    }

    /// List all keys matching a prefix
    pub fn list(&self, prefix: &str) -> Vec<String> {
        self.cleanup();

        self.data
            .iter()
            .filter(|entry| entry.key().starts_with(prefix))
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get metadata for a key
    pub fn metadata(&self, key: &str) -> Result<StorageMetadata> {
        self.data
            .get(key)
            .map(|entry| entry.metadata.clone())
            .ok_or_else(|| AnalyticsError::Storage(format!("Key not found: {}", key)))
    }

    /// Get the total number of stored entries
    pub fn size(&self) -> usize {
        self.cleanup();
        self.data.len()
    }

    /// Clear all data
    pub fn clear(&self) {
        self.data.clear();
    }

    /// Export all data to a file
    pub fn export(&self, path: impl AsRef<Path>) -> Result<()> {
        let entries: Vec<StorageEntry> = self.data.iter().map(|e| e.value().clone()).collect();

        let json = serde_json::to_string_pretty(&entries)?;
        std::fs::write(path, json)?;

        Ok(())
    }

    /// Import data from a file
    pub fn import(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = std::fs::read_to_string(path)?;
        let entries: Vec<StorageEntry> = serde_json::from_str(&json)?;

        for entry in entries {
            self.data.insert(entry.key.clone(), entry);
        }

        Ok(())
    }

    fn cleanup(&self) {
        if self.retention_seconds <= 0 {
            return;
        }

        let cutoff = Utc::now() - chrono::Duration::seconds(self.retention_seconds);
        self.data.retain(|_, entry| entry.timestamp > cutoff);
    }
}

/// Time-series storage optimized for analytics
pub struct TimeSeriesStorage {
    series: Arc<DashMap<String, Vec<TimePoint>>>,
    max_points_per_series: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub tags: Vec<(String, String)>,
}

impl TimeSeriesStorage {
    pub fn new(max_points_per_series: usize) -> Self {
        Self {
            series: Arc::new(DashMap::new()),
            max_points_per_series,
        }
    }

    /// Add a point to a time series
    pub fn add_point(&self, series_name: impl Into<String>, point: TimePoint) {
        let series_name = series_name.into();

        self.series
            .entry(series_name)
            .or_insert_with(Vec::new)
            .push(point);

        // Trim to max points
        let mut series = self.series.get_mut(&series_name.into()).unwrap();
        if series.len() > self.max_points_per_series {
            series.drain(0..series.len() - self.max_points_per_series);
        }
    }

    /// Get all points in a time series
    pub fn get_series(&self, series_name: &str) -> Option<Vec<TimePoint>> {
        self.series.get(series_name).map(|s| s.clone())
    }

    /// Get points in a time range
    pub fn get_range(
        &self,
        series_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<TimePoint> {
        self.series
            .get(series_name)
            .map(|series| {
                series
                    .iter()
                    .filter(|p| p.timestamp >= start && p.timestamp <= end)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// List all series names
    pub fn list_series(&self) -> Vec<String> {
        self.series.iter().map(|e| e.key().clone()).collect()
    }

    /// Delete a time series
    pub fn delete_series(&self, series_name: &str) {
        self.series.remove(series_name);
    }

    /// Clear all data
    pub fn clear(&self) {
        self.series.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage() {
        let storage = AnalyticsStorage::new(3600);

        let metadata = StorageMetadata {
            content_type: "text/plain".to_string(),
            size: 5,
            tags: vec![],
        };

        storage.put("test", b"hello".to_vec(), metadata).unwrap();

        let data = storage.get("test").unwrap();
        assert_eq!(data, b"hello");

        assert!(storage.exists("test"));
        storage.delete("test").unwrap();
        assert!(!storage.exists("test"));
    }

    #[test]
    fn test_timeseries_storage() {
        let storage = TimeSeriesStorage::new(100);

        let point = TimePoint {
            timestamp: Utc::now(),
            value: 42.0,
            tags: vec![],
        };

        storage.add_point("metric1", point);

        let series = storage.get_series("metric1").unwrap();
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].value, 42.0);
    }
}
