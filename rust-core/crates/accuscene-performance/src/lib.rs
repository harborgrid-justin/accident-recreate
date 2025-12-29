//! AccuScene Performance - High-Performance Streaming Engine
//!
//! This crate provides a comprehensive suite of performance optimization utilities including:
//! - Zero-copy streaming with backpressure control
//! - Advanced memory management (arena, slab, object pooling)
//! - Lock-free concurrency primitives
//! - Work-stealing thread pool
//! - CPU and memory profiling
//! - Latency tracking with histograms
//! - SIMD optimizations
//! - Cache-aligned data structures
//! - Comprehensive metrics collection
//!
//! # Features
//!
//! - **Zero-allocation hot paths**: Minimize allocations in critical code paths
//! - **Sub-millisecond latency**: Optimized for ultra-low latency operations
//! - **SIMD support**: Vector operations for data processing
//! - **Cache-friendly layouts**: Optimized memory layouts for CPU cache
//! - **Metrics collection**: Prometheus-compatible metrics export
//!
//! # Example
//!
//! ```rust,no_run
//! use accuscene_performance::prelude::*;
//!
//! // Create a streaming pipeline
//! let mut pipeline = Pipeline::builder()
//!     .buffer_size(8192)
//!     .backpressure_strategy(BackpressureStrategy::Block)
//!     .build();
//!
//! // Create a memory pool
//! let pool = BufferPool::new(100, 4096);
//!
//! // Use lock-free data structures
//! let queue = LockFreeQueue::new();
//! queue.push(42);
//!
//! // Collect metrics
//! let counter = counter("requests_total");
//! counter.inc();
//! ```

#![warn(missing_docs)]
#![allow(dead_code)]

pub mod config;
pub mod error;

pub mod streaming;
pub mod memory;
pub mod concurrency;
pub mod profiling;
pub mod optimization;
pub mod metrics;

pub use config::PerformanceConfig;
pub use error::{PerformanceError, Result};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::config::PerformanceConfig;
    pub use crate::error::{PerformanceError, Result};

    // Streaming
    pub use crate::streaming::{
        BackpressureStrategy, Pipeline, PipelineBuilder, Sink, Source, StreamItem, StreamStats,
        Transform, Window,
    };

    // Memory
    pub use crate::memory::{
        Arena, ArenaAllocator, BufferPool, ObjectPool, Slab, SlabAllocator, ZeroCopyBuffer,
        ZeroCopySlice,
    };

    // Concurrency
    pub use crate::concurrency::{
        AtomicBatch, LockFreeQueue, LockFreeStack, WorkStealingPool,
        bounded, unbounded, Receiver, Sender,
    };

    // Profiling
    pub use crate::profiling::{
        CpuProfiler, FlamegraphGenerator, LatencyHistogram, LatencyTracker, MemoryProfiler,
        Timer,
    };

    // Optimization
    pub use crate::optimization::{
        CacheAligned, CachePadded, prefetch_read, prefetch_write, simd_add, simd_sum,
    };

    // Metrics
    pub use crate::metrics::{
        Counter, Histogram, MetricsRegistry, PrometheusReporter, counter, histogram,
        global_registry,
    };
}

/// Performance runtime for managing resources
pub struct PerformanceRuntime {
    config: PerformanceConfig,
    metrics_registry: metrics::MetricsRegistry,
    profiler_cpu: profiling::CpuProfiler,
    profiler_memory: profiling::MemoryProfiler,
}

impl PerformanceRuntime {
    /// Create a new performance runtime
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            metrics_registry: metrics::MetricsRegistry::new(),
            profiler_cpu: profiling::CpuProfiler::new(),
            profiler_memory: profiling::MemoryProfiler::new(),
        }
    }

    /// Create with default configuration
    pub fn default_runtime() -> Self {
        Self::new(PerformanceConfig::default())
    }

    /// Get configuration
    pub fn config(&self) -> &PerformanceConfig {
        &self.config
    }

    /// Get metrics registry
    pub fn metrics(&self) -> &metrics::MetricsRegistry {
        &self.metrics_registry
    }

    /// Get CPU profiler
    pub fn cpu_profiler(&mut self) -> &mut profiling::CpuProfiler {
        &mut self.profiler_cpu
    }

    /// Get memory profiler
    pub fn memory_profiler(&mut self) -> &mut profiling::MemoryProfiler {
        &mut self.profiler_memory
    }

    /// Start all profiling
    pub fn start_profiling(&mut self) {
        use crate::profiling::Profiler;
        if self.config.profiling.enable_cpu {
            self.profiler_cpu.start();
        }
        if self.config.profiling.enable_memory {
            self.profiler_memory.start();
        }
    }

    /// Stop all profiling
    pub fn stop_profiling(&mut self) {
        use crate::profiling::Profiler;
        self.profiler_cpu.stop();
        self.profiler_memory.stop();
    }

    /// Print profiling results
    pub fn print_profiling_results(&self) {
        if self.config.profiling.enable_cpu {
            self.profiler_cpu.print_stats();
        }
        if self.config.profiling.enable_memory {
            self.profiler_memory.print_stats();
        }
    }

    /// Print metrics
    pub fn print_metrics(&self) {
        self.metrics_registry.print();
    }

    /// Get Prometheus metrics
    pub fn prometheus_metrics(&self) -> String {
        let reporter = metrics::PrometheusReporter::default();
        reporter.to_prometheus(&self.metrics_registry)
    }
}

impl Default for PerformanceRuntime {
    fn default() -> Self {
        Self::default_runtime()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = PerformanceRuntime::default();
        assert!(runtime.config().streaming.buffer_size > 0);
    }

    #[test]
    fn test_runtime_metrics() {
        let runtime = PerformanceRuntime::default();
        let counter = runtime.metrics().counter("test");
        counter.inc();
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_profiling_lifecycle() {
        let mut runtime = PerformanceRuntime::default();

        runtime.start_profiling();
        assert!(runtime.cpu_profiler().is_active());
        assert!(runtime.memory_profiler().is_active());

        runtime.stop_profiling();
        assert!(!runtime.cpu_profiler().is_active());
        assert!(!runtime.memory_profiler().is_active());
    }
}
