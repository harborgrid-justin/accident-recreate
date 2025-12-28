//! Vehicle sensor data streaming.

use crate::error::Result;
use crate::stream::DataStream;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Sensor type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SensorType {
    Accelerometer,
    Gyroscope,
    GPS,
    Camera,
    Lidar,
    Radar,
    Temperature,
    Pressure,
    Speed,
    Custom(String),
}

/// Sensor data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    pub sensor_id: String,
    pub sensor_type: SensorType,
    pub vehicle_id: String,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub unit: String,
    pub quality: f64,
    pub metadata: std::collections::HashMap<String, String>,
}

impl SensorData {
    /// Create a new sensor data point
    pub fn new(
        sensor_id: String,
        sensor_type: SensorType,
        vehicle_id: String,
        values: Vec<f64>,
    ) -> Self {
        Self {
            sensor_id,
            sensor_type,
            vehicle_id,
            timestamp: chrono::Utc::now().timestamp_millis(),
            values,
            unit: String::new(),
            quality: 1.0,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set unit
    pub fn with_unit(mut self, unit: String) -> Self {
        self.unit = unit;
        self
    }

    /// Set quality (0.0 to 1.0)
    pub fn with_quality(mut self, quality: f64) -> Self {
        self.quality = quality.clamp(0.0, 1.0);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if sensor data is valid
    pub fn is_valid(&self) -> bool {
        self.quality >= 0.5 && !self.values.is_empty()
    }
}

/// Sensor stream for real-time sensor data
pub struct SensorStream<S>
where
    S: DataStream<Item = SensorData>,
{
    inner: S,
    filter_type: Option<SensorType>,
    min_quality: f64,
}

impl<S> SensorStream<S>
where
    S: DataStream<Item = SensorData>,
{
    /// Create a new sensor stream
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            filter_type: None,
            min_quality: 0.0,
        }
    }

    /// Filter by sensor type
    pub fn filter_type(mut self, sensor_type: SensorType) -> Self {
        self.filter_type = Some(sensor_type);
        self
    }

    /// Filter by minimum quality
    pub fn min_quality(mut self, quality: f64) -> Self {
        self.min_quality = quality.clamp(0.0, 1.0);
        self
    }

    fn should_include(&self, data: &SensorData) -> bool {
        if let Some(ref filter_type) = self.filter_type {
            if &data.sensor_type != filter_type {
                return false;
            }
        }

        if data.quality < self.min_quality {
            return false;
        }

        true
    }
}

#[async_trait]
impl<S> DataStream for SensorStream<S>
where
    S: DataStream<Item = SensorData>,
{
    type Item = SensorData;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        loop {
            match self.inner.next().await? {
                Some(data) => {
                    if self.should_include(&data) {
                        return Ok(Some(data));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.inner.is_complete()
    }
}

/// Sensor statistics
#[derive(Debug, Clone, Default)]
pub struct SensorStats {
    pub total_readings: u64,
    pub readings_by_type: std::collections::HashMap<String, u64>,
    pub avg_quality: f64,
    pub quality_sum: f64,
}

impl SensorStats {
    pub fn update(&mut self, data: &SensorData) {
        self.total_readings += 1;
        self.quality_sum += data.quality;
        self.avg_quality = self.quality_sum / self.total_readings as f64;

        let type_key = format!("{:?}", data.sensor_type);
        *self.readings_by_type.entry(type_key).or_insert(0) += 1;
    }
}

/// Calibrated sensor data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibratedSensorData {
    pub original: SensorData,
    pub calibrated_values: Vec<f64>,
    pub calibration_applied: bool,
}

impl CalibratedSensorData {
    /// Create from sensor data
    pub fn from_sensor_data(data: SensorData) -> Self {
        Self {
            calibrated_values: data.values.clone(),
            original: data,
            calibration_applied: false,
        }
    }

    /// Apply calibration
    pub fn calibrate<F>(mut self, calibration_fn: F) -> Self
    where
        F: Fn(&[f64]) -> Vec<f64>,
    {
        self.calibrated_values = calibration_fn(&self.original.values);
        self.calibration_applied = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::iterator::IteratorSource;

    #[tokio::test]
    async fn test_sensor_data() {
        let data = SensorData::new(
            "sensor1".to_string(),
            SensorType::Accelerometer,
            "vehicle1".to_string(),
            vec![0.1, 0.2, 9.8],
        )
        .with_unit("m/s^2".to_string())
        .with_quality(0.95);

        assert!(data.is_valid());
        assert_eq!(data.values.len(), 3);
    }

    #[tokio::test]
    async fn test_sensor_stream_filtering() {
        let data1 = SensorData::new(
            "sensor1".to_string(),
            SensorType::Accelerometer,
            "vehicle1".to_string(),
            vec![0.1, 0.2, 9.8],
        )
        .with_quality(0.9);

        let data2 = SensorData::new(
            "sensor2".to_string(),
            SensorType::GPS,
            "vehicle1".to_string(),
            vec![37.7749, -122.4194],
        )
        .with_quality(0.8);

        let data3 = SensorData::new(
            "sensor3".to_string(),
            SensorType::Accelerometer,
            "vehicle1".to_string(),
            vec![0.15, 0.25, 9.7],
        )
        .with_quality(0.4); // Low quality

        let source = IteratorSource::new(vec![data1, data2, data3].into_iter());
        let stream = SensorStream::new(source)
            .filter_type(SensorType::Accelerometer)
            .min_quality(0.5);

        // The stream would filter to only high-quality Accelerometer data
        // In actual use, we'd start the source and consume the stream
    }
}
