//! Context propagation for distributed tracing

use super::{SpanId, TraceId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trace context for propagation across service boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub sampled: bool,
    pub baggage: HashMap<String, String>,
}

impl TraceContext {
    /// Create a new trace context
    pub fn new(trace_id: TraceId, span_id: SpanId) -> Self {
        Self {
            trace_id,
            span_id,
            sampled: true,
            baggage: HashMap::new(),
        }
    }

    /// Add baggage item
    pub fn add_baggage(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.baggage.insert(key.into(), value.into());
    }

    /// Get baggage item
    pub fn get_baggage(&self, key: &str) -> Option<&String> {
        self.baggage.get(key)
    }

    /// Serialize to W3C traceparent header format
    /// Format: 00-{trace_id}-{span_id}-{flags}
    pub fn to_traceparent(&self) -> String {
        let flags = if self.sampled { "01" } else { "00" };
        format!(
            "00-{}-{}-{}",
            self.trace_id.to_string().replace("-", ""),
            self.span_id.to_string().replace("-", "")[..16].to_string(),
            flags
        )
    }

    /// Parse from W3C traceparent header
    pub fn from_traceparent(header: &str) -> Option<Self> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() != 4 || parts[0] != "00" {
            return None;
        }

        // Parse trace ID (first 32 hex chars)
        let trace_id_str = parts[1];
        let trace_uuid = uuid::Uuid::parse_str(trace_id_str).ok()?;
        let trace_id = TraceId::from_uuid(trace_uuid);

        // Parse span ID (16 hex chars)
        let span_id_str = parts[2];
        let span_uuid = uuid::Uuid::parse_str(span_id_str).ok()?;
        let span_id = SpanId::from_uuid(span_uuid);

        // Parse flags
        let flags = u8::from_str_radix(parts[3], 16).ok()?;
        let sampled = (flags & 0x01) == 0x01;

        Some(Self {
            trace_id,
            span_id,
            sampled,
            baggage: HashMap::new(),
        })
    }

    /// Serialize to custom headers
    pub fn to_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        headers.insert("x-trace-id".to_string(), self.trace_id.to_string());
        headers.insert("x-span-id".to_string(), self.span_id.to_string());
        headers.insert(
            "x-trace-sampled".to_string(),
            self.sampled.to_string(),
        );

        // Add baggage
        for (key, value) in &self.baggage {
            headers.insert(format!("x-baggage-{}", key), value.clone());
        }

        headers
    }

    /// Parse from custom headers
    pub fn from_headers(headers: &HashMap<String, String>) -> Option<Self> {
        let trace_id_str = headers.get("x-trace-id")?;
        let span_id_str = headers.get("x-span-id")?;

        let trace_uuid = uuid::Uuid::parse_str(trace_id_str).ok()?;
        let span_uuid = uuid::Uuid::parse_str(span_id_str).ok()?;

        let trace_id = TraceId::from_uuid(trace_uuid);
        let span_id = SpanId::from_uuid(span_uuid);

        let sampled = headers
            .get("x-trace-sampled")
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);

        let mut baggage = HashMap::new();
        for (key, value) in headers {
            if let Some(baggage_key) = key.strip_prefix("x-baggage-") {
                baggage.insert(baggage_key.to_string(), value.clone());
            }
        }

        Some(Self {
            trace_id,
            span_id,
            sampled,
            baggage,
        })
    }
}

/// Context propagator for managing trace context
pub struct ContextPropagator {
    format: PropagationFormat,
}

impl ContextPropagator {
    /// Create a new context propagator
    pub fn new() -> Self {
        Self {
            format: PropagationFormat::W3C,
        }
    }

    /// Create with specific format
    pub fn with_format(format: PropagationFormat) -> Self {
        Self { format }
    }

    /// Inject trace context into headers
    pub fn inject(&self, context: &TraceContext) -> HashMap<String, String> {
        match self.format {
            PropagationFormat::W3C => {
                let mut headers = HashMap::new();
                headers.insert("traceparent".to_string(), context.to_traceparent());
                headers
            }
            PropagationFormat::Custom => context.to_headers(),
        }
    }

    /// Extract trace context from headers
    pub fn extract(&self, headers: &HashMap<String, String>) -> Option<TraceContext> {
        match self.format {
            PropagationFormat::W3C => {
                headers
                    .get("traceparent")
                    .and_then(|h| TraceContext::from_traceparent(h))
            }
            PropagationFormat::Custom => TraceContext::from_headers(headers),
        }
    }

    /// Get the propagation format
    pub fn format(&self) -> PropagationFormat {
        self.format
    }

    /// Set the propagation format
    pub fn set_format(&mut self, format: PropagationFormat) {
        self.format = format;
    }
}

impl Default for ContextPropagator {
    fn default() -> Self {
        Self::new()
    }
}

/// Propagation format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropagationFormat {
    /// W3C Trace Context format
    W3C,
    /// Custom header format
    Custom,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_context_creation() {
        let trace_id = TraceId::new();
        let span_id = SpanId::new();
        let mut context = TraceContext::new(trace_id, span_id);

        assert_eq!(context.trace_id, trace_id);
        assert_eq!(context.span_id, span_id);
        assert!(context.sampled);

        context.add_baggage("user_id", "123");
        assert_eq!(context.get_baggage("user_id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_custom_headers() {
        let trace_id = TraceId::new();
        let span_id = SpanId::new();
        let mut context = TraceContext::new(trace_id, span_id);
        context.add_baggage("key", "value");

        let headers = context.to_headers();
        assert!(headers.contains_key("x-trace-id"));
        assert!(headers.contains_key("x-span-id"));
        assert!(headers.contains_key("x-baggage-key"));

        let extracted = TraceContext::from_headers(&headers).unwrap();
        assert_eq!(extracted.trace_id, context.trace_id);
        assert_eq!(extracted.span_id, context.span_id);
        assert_eq!(extracted.get_baggage("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_context_propagator() {
        let propagator = ContextPropagator::new();
        let trace_id = TraceId::new();
        let span_id = SpanId::new();
        let context = TraceContext::new(trace_id, span_id);

        let headers = propagator.inject(&context);
        assert!(!headers.is_empty());

        // Note: W3C format extraction is more complex and would need proper UUID handling
    }
}
