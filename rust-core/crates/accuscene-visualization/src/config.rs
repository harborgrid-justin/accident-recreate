use serde::{Deserialize, Serialize};

/// Configuration for chart rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// Chart width in pixels
    pub width: u32,

    /// Chart height in pixels
    pub height: u32,

    /// Margin configuration
    pub margin: Margin,

    /// Color scheme
    pub colors: ColorScheme,

    /// Font configuration
    pub font: FontConfig,

    /// Animation settings
    pub animation: AnimationConfig,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            margin: Margin::default(),
            colors: ColorScheme::default(),
            font: FontConfig::default(),
            animation: AnimationConfig::default(),
        }
    }
}

/// Chart margin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margin {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Default for Margin {
    fn default() -> Self {
        Self {
            top: 20,
            right: 20,
            bottom: 40,
            left: 60,
        }
    }
}

/// Color scheme for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    /// Primary colors for data series
    pub primary: Vec<String>,

    /// Background color
    pub background: String,

    /// Text color
    pub text: String,

    /// Grid line color
    pub grid: String,

    /// Axis color
    pub axis: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            primary: vec![
                "#3B82F6".to_string(), // Blue
                "#10B981".to_string(), // Green
                "#F59E0B".to_string(), // Amber
                "#EF4444".to_string(), // Red
                "#8B5CF6".to_string(), // Purple
                "#EC4899".to_string(), // Pink
                "#06B6D4".to_string(), // Cyan
                "#84CC16".to_string(), // Lime
            ],
            background: "#FFFFFF".to_string(),
            text: "#1F2937".to_string(),
            grid: "#E5E7EB".to_string(),
            axis: "#6B7280".to_string(),
        }
    }
}

/// Font configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: u32,
    pub weight: String,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "Inter, sans-serif".to_string(),
            size: 12,
            weight: "normal".to_string(),
        }
    }
}

/// Animation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Enable animations
    pub enabled: bool,

    /// Animation duration in milliseconds
    pub duration: u32,

    /// Easing function
    pub easing: String,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: 300,
            easing: "ease-in-out".to_string(),
        }
    }
}

/// Data aggregation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    /// Aggregation method
    pub method: AggregationMethod,

    /// Window size for rolling aggregations
    pub window_size: Option<usize>,

    /// Number of bins for histograms
    pub bin_count: Option<usize>,
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            method: AggregationMethod::Mean,
            window_size: None,
            bin_count: Some(20),
        }
    }
}

/// Aggregation methods
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AggregationMethod {
    Mean,
    Median,
    Sum,
    Min,
    Max,
    Count,
    StdDev,
    Variance,
}

/// Interpolation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpolationConfig {
    /// Interpolation method
    pub method: InterpolationMethod,

    /// Number of points to generate
    pub point_count: usize,

    /// Smoothing factor (0.0 to 1.0)
    pub smoothing: f64,
}

impl Default for InterpolationConfig {
    fn default() -> Self {
        Self {
            method: InterpolationMethod::Linear,
            point_count: 100,
            smoothing: 0.5,
        }
    }
}

/// Interpolation methods
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InterpolationMethod {
    Linear,
    Cubic,
    Spline,
    Step,
    Basis,
}

/// Sampling configuration for large datasets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    /// Maximum number of points to display
    pub max_points: usize,

    /// Sampling strategy
    pub strategy: SamplingStrategy,

    /// Preserve outliers
    pub preserve_outliers: bool,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            max_points: 1000,
            strategy: SamplingStrategy::LTTB,
            preserve_outliers: true,
        }
    }
}

/// Sampling strategies
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SamplingStrategy {
    /// Largest Triangle Three Buckets algorithm
    LTTB,

    /// Uniform random sampling
    Random,

    /// Systematic sampling (every nth point)
    Systematic,

    /// Min-Max sampling (preserve peaks and valleys)
    MinMax,
}
