//! Stream processing utilities for events.

pub mod aggregate;
pub mod filter;
pub mod transform;

pub use aggregate::{Aggregator, WindowAggregator};
pub use filter::{EventFilterStream, FilterPredicate};
pub use transform::{EventTransformer, TransformStream};

use crate::event::Event;
use futures::Stream;
use std::pin::Pin;

/// Type alias for event stream
pub type EventStream = Pin<Box<dyn Stream<Item = Event> + Send>>;
