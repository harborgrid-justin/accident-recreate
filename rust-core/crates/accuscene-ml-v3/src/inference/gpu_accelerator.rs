//! GPU acceleration utilities and management

use crate::error::{MlError, Result};
use crate::inference::Device;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// GPU device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// Device ID
    pub device_id: u32,

    /// Device name
    pub name: String,

    /// Total memory in bytes
    pub total_memory: u64,

    /// Free memory in bytes
    pub free_memory: u64,

    /// Compute capability (major, minor)
    pub compute_capability: (u32, u32),

    /// CUDA version (if applicable)
    pub cuda_version: Option<String>,

    /// Is device available
    pub available: bool,
}

/// GPU utilization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuUtilization {
    /// GPU device ID
    pub device_id: u32,

    /// GPU utilization percentage (0-100)
    pub gpu_percent: f32,

    /// Memory utilization percentage (0-100)
    pub memory_percent: f32,

    /// Memory used in bytes
    pub memory_used: u64,

    /// Temperature in Celsius
    pub temperature: Option<f32>,

    /// Power usage in watts
    pub power_usage: Option<f32>,
}

/// GPU allocator for managing GPU memory
pub struct GpuAllocator {
    device_id: u32,
    total_memory: u64,
    allocated_memory: Arc<RwLock<u64>>,
    max_memory_limit: u64,
}

impl GpuAllocator {
    /// Create a new GPU allocator
    pub fn new(device_id: u32, total_memory: u64, max_memory_limit: u64) -> Self {
        Self {
            device_id,
            total_memory,
            allocated_memory: Arc::new(RwLock::new(0)),
            max_memory_limit,
        }
    }

    /// Allocate GPU memory
    pub async fn allocate(&self, size: u64) -> Result<GpuAllocation> {
        let mut allocated = self.allocated_memory.write().await;

        if *allocated + size > self.max_memory_limit {
            return Err(MlError::Gpu(format!(
                "Insufficient GPU memory: requested {}, available {}",
                size,
                self.max_memory_limit - *allocated
            )));
        }

        *allocated += size;

        Ok(GpuAllocation {
            device_id: self.device_id,
            size,
            allocator: Arc::clone(&self.allocated_memory),
        })
    }

    /// Get current memory usage
    pub async fn memory_used(&self) -> u64 {
        *self.allocated_memory.read().await
    }

    /// Get available memory
    pub async fn memory_available(&self) -> u64 {
        let used = self.memory_used().await;
        self.max_memory_limit.saturating_sub(used)
    }

    /// Get memory utilization percentage
    pub async fn memory_utilization(&self) -> f32 {
        let used = self.memory_used().await;
        (used as f32 / self.total_memory as f32) * 100.0
    }
}

/// GPU memory allocation
pub struct GpuAllocation {
    device_id: u32,
    size: u64,
    allocator: Arc<RwLock<u64>>,
}

impl Drop for GpuAllocation {
    fn drop(&mut self) {
        // Free memory when allocation is dropped
        let allocator = Arc::clone(&self.allocator);
        let size = self.size;

        tokio::spawn(async move {
            let mut allocated = allocator.write().await;
            *allocated = allocated.saturating_sub(size);
        });
    }
}

/// GPU accelerator for managing GPU resources
pub struct GpuAccelerator {
    devices: Vec<GpuInfo>,
    allocators: Vec<Arc<GpuAllocator>>,
    current_device: Arc<RwLock<usize>>,
}

impl GpuAccelerator {
    /// Create a new GPU accelerator
    pub fn new() -> Result<Self> {
        let devices = Self::detect_gpus()?;

        if devices.is_empty() {
            return Err(MlError::ResourceUnavailable(
                "No GPU devices detected".to_string(),
            ));
        }

        let allocators = devices
            .iter()
            .map(|info| {
                let limit = (info.total_memory as f64 * 0.9) as u64; // Use 90% max
                Arc::new(GpuAllocator::new(info.device_id, info.total_memory, limit))
            })
            .collect();

        Ok(Self {
            devices,
            allocators,
            current_device: Arc::new(RwLock::new(0)),
        })
    }

    /// Detect available GPUs
    fn detect_gpus() -> Result<Vec<GpuInfo>> {
        #[cfg(feature = "gpu")]
        {
            // In a real implementation, this would query CUDA/ROCm/etc.
            // For now, return a simulated GPU if the feature is enabled
            Ok(vec![GpuInfo {
                device_id: 0,
                name: "NVIDIA GPU (Simulated)".to_string(),
                total_memory: 8 * 1024 * 1024 * 1024, // 8 GB
                free_memory: 6 * 1024 * 1024 * 1024,   // 6 GB
                compute_capability: (8, 0),
                cuda_version: Some("12.0".to_string()),
                available: true,
            }])
        }

        #[cfg(not(feature = "gpu"))]
        {
            Ok(vec![])
        }
    }

    /// Get list of available GPUs
    pub fn list_devices(&self) -> &[GpuInfo] {
        &self.devices
    }

    /// Get specific GPU info
    pub fn get_device(&self, device_id: u32) -> Option<&GpuInfo> {
        self.devices
            .iter()
            .find(|d| d.device_id == device_id)
    }

    /// Select GPU device
    pub async fn select_device(&self, device_id: u32) -> Result<()> {
        let device_exists = self.devices.iter().any(|d| d.device_id == device_id);

        if !device_exists {
            return Err(MlError::Gpu(format!(
                "GPU device {} not found",
                device_id
            )));
        }

        let mut current = self.current_device.write().await;
        *current = device_id as usize;

        tracing::info!("Selected GPU device {}", device_id);
        Ok(())
    }

    /// Get current GPU device
    pub async fn current_device(&self) -> u32 {
        *self.current_device.read().await as u32
    }

    /// Allocate memory on current GPU
    pub async fn allocate(&self, size: u64) -> Result<GpuAllocation> {
        let device_idx = *self.current_device.read().await;
        let allocator = &self.allocators[device_idx];
        allocator.allocate(size).await
    }

    /// Get GPU utilization stats
    pub async fn get_utilization(&self, device_id: u32) -> Result<GpuUtilization> {
        let device_idx = device_id as usize;

        if device_idx >= self.allocators.len() {
            return Err(MlError::Gpu(format!(
                "Invalid device ID: {}",
                device_id
            )));
        }

        let allocator = &self.allocators[device_idx];
        let memory_used = allocator.memory_used().await;
        let total_memory = self.devices[device_idx].total_memory;

        Ok(GpuUtilization {
            device_id,
            gpu_percent: 0.0, // Would be queried from GPU in real implementation
            memory_percent: (memory_used as f32 / total_memory as f32) * 100.0,
            memory_used,
            temperature: None,
            power_usage: None,
        })
    }

    /// Get utilization for all GPUs
    pub async fn get_all_utilization(&self) -> Result<Vec<GpuUtilization>> {
        let mut utilizations = Vec::new();

        for device in &self.devices {
            let util = self.get_utilization(device.device_id).await?;
            utilizations.push(util);
        }

        Ok(utilizations)
    }

    /// Select best available GPU based on free memory
    pub async fn select_best_device(&self) -> Result<u32> {
        let mut best_device = 0;
        let mut max_free_memory = 0;

        for (idx, allocator) in self.allocators.iter().enumerate() {
            let free = allocator.memory_available().await;
            if free > max_free_memory {
                max_free_memory = free;
                best_device = idx;
            }
        }

        self.select_device(best_device as u32).await?;
        Ok(best_device as u32)
    }

    /// Check if GPU is available
    pub fn is_available(&self) -> bool {
        !self.devices.is_empty() && self.devices.iter().any(|d| d.available)
    }

    /// Get total GPU count
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
}

impl Default for GpuAccelerator {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Return empty accelerator if GPU initialization fails
            Self {
                devices: vec![],
                allocators: vec![],
                current_device: Arc::new(RwLock::new(0)),
            }
        })
    }
}

/// GPU pool for load balancing across multiple GPUs
pub struct GpuPool {
    accelerator: Arc<GpuAccelerator>,
    round_robin_index: Arc<RwLock<usize>>,
}

impl GpuPool {
    /// Create a new GPU pool
    pub fn new(accelerator: Arc<GpuAccelerator>) -> Self {
        Self {
            accelerator,
            round_robin_index: Arc::new(RwLock::new(0)),
        }
    }

    /// Get next GPU in round-robin fashion
    pub async fn next_device(&self) -> u32 {
        let device_count = self.accelerator.device_count();

        if device_count == 0 {
            return 0;
        }

        let mut index = self.round_robin_index.write().await;
        let device_id = *index as u32;
        *index = (*index + 1) % device_count;

        device_id
    }

    /// Get least loaded GPU
    pub async fn least_loaded_device(&self) -> Result<u32> {
        let utilizations = self.accelerator.get_all_utilization().await?;

        utilizations
            .iter()
            .min_by(|a, b| {
                a.memory_percent
                    .partial_cmp(&b.memory_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|u| u.device_id)
            .ok_or_else(|| MlError::Gpu("No GPUs available".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_allocator() {
        let allocator = GpuAllocator::new(0, 1024 * 1024 * 1024, 512 * 1024 * 1024);

        let alloc1 = allocator.allocate(100 * 1024 * 1024).await.unwrap();
        assert_eq!(allocator.memory_used().await, 100 * 1024 * 1024);

        drop(alloc1);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        assert_eq!(allocator.memory_used().await, 0);
    }

    #[test]
    fn test_gpu_info() {
        let info = GpuInfo {
            device_id: 0,
            name: "Test GPU".to_string(),
            total_memory: 8 * 1024 * 1024 * 1024,
            free_memory: 6 * 1024 * 1024 * 1024,
            compute_capability: (8, 0),
            cuda_version: Some("12.0".to_string()),
            available: true,
        };

        assert_eq!(info.device_id, 0);
        assert!(info.available);
    }

    #[test]
    fn test_gpu_utilization() {
        let util = GpuUtilization {
            device_id: 0,
            gpu_percent: 75.0,
            memory_percent: 60.0,
            memory_used: 4 * 1024 * 1024 * 1024,
            temperature: Some(65.0),
            power_usage: Some(250.0),
        };

        assert_eq!(util.gpu_percent, 75.0);
        assert_eq!(util.memory_percent, 60.0);
    }
}
