//! Stream operators for transforming and combining data streams.

pub mod aggregate;
pub mod filter;
pub mod flatmap;
pub mod join;
pub mod keyby;
pub mod map;
pub mod window;

pub use self::aggregate::{AggregateOperator, Aggregator};
pub use self::filter::FilterOperator;
pub use self::flatmap::FlatMapOperator;
pub use self::join::{JoinOperator, JoinType};
pub use self::keyby::{KeyByOperator, KeyExtractor};
pub use self::map::MapOperator;
pub use self::window::{WindowAssigner, WindowOperator, WindowType};
