//! Statistical analysis framework

pub mod correlation;
pub mod descriptive;
pub mod distribution;
pub mod regression;

pub use correlation::{CorrelationAnalyzer, CorrelationType};
pub use descriptive::{DescriptiveStats, Statistics};
pub use distribution::{DistributionFitter, DistributionType};
pub use regression::{LinearRegression, PolynomialRegression, RegressionResult};

use crate::error::Result;
