//! Service registry for dependency injection
//!
//! This module provides a service registry that manages all registered
//! services and allows for dependency injection.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

/// Service descriptor containing information about a registered service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDescriptor {
    /// Service name
    pub name: String,

    /// Service description
    pub description: String,

    /// Service version
    pub version: String,

    /// Service status
    pub status: ServiceStatus,

    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Service metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Service status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    /// Service is initializing
    Initializing,
    /// Service is running
    Running,
    /// Service is stopped
    Stopped,
    /// Service has an error
    Error,
}

/// Service registry for managing all services
pub struct Registry {
    services: Arc<DashMap<String, ServiceDescriptor>>,
}

impl Registry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self {
            services: Arc::new(DashMap::new()),
        }
    }

    /// Register a service
    pub fn register(&self, name: String, description: String, version: String) {
        let now = chrono::Utc::now();

        let descriptor = ServiceDescriptor {
            name: name.clone(),
            description,
            version,
            status: ServiceStatus::Initializing,
            registered_at: now,
            updated_at: now,
            metadata: std::collections::HashMap::new(),
        };

        self.services.insert(name.clone(), descriptor);
        info!("Service registered: {}", name);
    }

    /// Register a service with metadata
    pub fn register_with_metadata(
        &self,
        name: String,
        description: String,
        version: String,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) {
        let now = chrono::Utc::now();

        let descriptor = ServiceDescriptor {
            name: name.clone(),
            description,
            version,
            status: ServiceStatus::Initializing,
            registered_at: now,
            updated_at: now,
            metadata,
        };

        self.services.insert(name.clone(), descriptor);
        info!("Service registered with metadata: {}", name);
    }

    /// Unregister a service
    pub fn unregister(&self, name: &str) {
        if self.services.remove(name).is_some() {
            info!("Service unregistered: {}", name);
        }
    }

    /// Check if a service is registered
    pub fn is_registered(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }

    /// Get a service descriptor
    pub fn get(&self, name: &str) -> Option<ServiceDescriptor> {
        self.services.get(name).map(|entry| entry.clone())
    }

    /// Update service status
    pub fn update_status(&self, name: &str, status: ServiceStatus) {
        if let Some(mut descriptor) = self.services.get_mut(name) {
            descriptor.status = status;
            descriptor.updated_at = chrono::Utc::now();
            info!("Service {} status updated to {:?}", name, status);
        }
    }

    /// Update service metadata
    pub fn update_metadata(
        &self,
        name: &str,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) {
        if let Some(mut descriptor) = self.services.get_mut(name) {
            descriptor.metadata = metadata;
            descriptor.updated_at = chrono::Utc::now();
        }
    }

    /// Add metadata entry
    pub fn add_metadata(&self, name: &str, key: String, value: serde_json::Value) {
        if let Some(mut descriptor) = self.services.get_mut(name) {
            descriptor.metadata.insert(key, value);
            descriptor.updated_at = chrono::Utc::now();
        }
    }

    /// Get metadata entry
    pub fn get_metadata(&self, name: &str, key: &str) -> Option<serde_json::Value> {
        self.services
            .get(name)
            .and_then(|descriptor| descriptor.metadata.get(key).cloned())
    }

    /// List all registered services
    pub fn list_services(&self) -> Vec<String> {
        self.services
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get all service descriptors
    pub fn list_descriptors(&self) -> Vec<ServiceDescriptor> {
        self.services
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get service count
    pub fn service_count(&self) -> usize {
        self.services.len()
    }

    /// Get services by status
    pub fn services_by_status(&self, status: ServiceStatus) -> Vec<ServiceDescriptor> {
        self.services
            .iter()
            .filter(|entry| entry.value().status == status)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get running services
    pub fn running_services(&self) -> Vec<String> {
        self.services
            .iter()
            .filter(|entry| entry.value().status == ServiceStatus::Running)
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get stopped services
    pub fn stopped_services(&self) -> Vec<String> {
        self.services
            .iter()
            .filter(|entry| entry.value().status == ServiceStatus::Stopped)
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get services with errors
    pub fn error_services(&self) -> Vec<String> {
        self.services
            .iter()
            .filter(|entry| entry.value().status == ServiceStatus::Error)
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Clear all services
    pub fn clear(&self) {
        self.services.clear();
        info!("All services cleared from registry");
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        let total = self.service_count();
        let running = self.services_by_status(ServiceStatus::Running).len();
        let stopped = self.services_by_status(ServiceStatus::Stopped).len();
        let error = self.services_by_status(ServiceStatus::Error).len();
        let initializing = self.services_by_status(ServiceStatus::Initializing).len();

        RegistryStats {
            total,
            running,
            stopped,
            error,
            initializing,
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    /// Total services
    pub total: usize,
    /// Running services
    pub running: usize,
    /// Stopped services
    pub stopped: usize,
    /// Services with errors
    pub error: usize,
    /// Initializing services
    pub initializing: usize,
}

impl RegistryStats {
    /// Check if all services are healthy
    pub fn all_healthy(&self) -> bool {
        self.error == 0 && self.running == self.total
    }

    /// Get health percentage
    pub fn health_percentage(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.running as f64 / self.total as f64) * 100.0
    }
}
