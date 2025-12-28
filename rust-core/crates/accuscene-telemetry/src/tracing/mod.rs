//! Distributed tracing module

pub mod span;
pub mod propagation;

use crate::{TracingConfig, Result};
use parking_lot::RwLock;
use std::sync::Arc;

pub use span::{SpanContext, SpanId, TraceId};
pub use propagation::{ContextPropagator, TraceContext};

/// Distributed tracing system
pub struct TracingSystem {
    config: TracingConfig,
    propagator: Arc<RwLock<ContextPropagator>>,
    active_spans: Arc<RwLock<Vec<SpanContext>>>,
}

impl TracingSystem {
    /// Create a new tracing system
    pub fn new(config: &TracingConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            propagator: Arc::new(RwLock::new(ContextPropagator::new())),
            active_spans: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Create a new span
    pub fn start_span(&self, name: impl Into<String>) -> SpanContext {
        let span = SpanContext::new(name, &self.config.service_name);

        // Add to active spans
        self.active_spans.write().push(span.clone());

        span
    }

    /// End a span
    pub fn end_span(&self, span: &SpanContext) {
        span.end();

        // Remove from active spans
        self.active_spans
            .write()
            .retain(|s| s.span_id() != span.span_id());
    }

    /// Get the context propagator
    pub fn propagator(&self) -> Arc<RwLock<ContextPropagator>> {
        Arc::clone(&self.propagator)
    }

    /// Get active spans count
    pub fn active_spans_count(&self) -> usize {
        self.active_spans.read().len()
    }

    /// Check if a trace should be sampled
    pub fn should_sample(&self) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.config.sampling_rate
    }

    /// Get the service name
    pub fn service_name(&self) -> &str {
        &self.config.service_name
    }
}

// Add rand dependency for sampling
use rand;
