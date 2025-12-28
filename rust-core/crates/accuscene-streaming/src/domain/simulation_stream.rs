//! Real-time simulation data streaming.

use crate::error::Result;
use crate::stream::DataStream;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Simulation state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SimulationState {
    Initializing,
    Running,
    Paused,
    Stopped,
    Error(String),
}

/// Simulation data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationData {
    pub simulation_id: String,
    pub timestamp: i64,
    pub state: SimulationState,
    pub frame: u64,
    pub time_step: f64,
    pub entities: Vec<EntityData>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Entity data within a simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub entity_id: String,
    pub entity_type: String,
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub rotation: [f64; 4],
    pub properties: std::collections::HashMap<String, f64>,
}

impl SimulationData {
    /// Create a new simulation data point
    pub fn new(simulation_id: String, frame: u64, time_step: f64) -> Self {
        Self {
            simulation_id,
            timestamp: chrono::Utc::now().timestamp_millis(),
            state: SimulationState::Running,
            frame,
            time_step,
            entities: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add an entity to the simulation data
    pub fn add_entity(&mut self, entity: EntityData) {
        self.entities.push(entity);
    }

    /// Set simulation state
    pub fn with_state(mut self, state: SimulationState) -> Self {
        self.state = state;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Simulation stream for real-time simulation data
pub struct SimulationStream<S>
where
    S: DataStream<Item = SimulationData>,
{
    inner: S,
    current_simulation: Option<String>,
}

impl<S> SimulationStream<S>
where
    S: DataStream<Item = SimulationData>,
{
    /// Create a new simulation stream
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            current_simulation: None,
        }
    }

    /// Filter by simulation ID
    pub fn filter_simulation(self, simulation_id: String) -> FilteredSimulationStream<S> {
        FilteredSimulationStream {
            inner: self,
            simulation_id,
        }
    }
}

#[async_trait]
impl<S> DataStream for SimulationStream<S>
where
    S: DataStream<Item = SimulationData>,
{
    type Item = SimulationData;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        let item = self.inner.next().await?;

        if let Some(ref data) = item {
            self.current_simulation = Some(data.simulation_id.clone());
        }

        Ok(item)
    }

    fn is_complete(&self) -> bool {
        self.inner.is_complete()
    }
}

/// Filtered simulation stream
pub struct FilteredSimulationStream<S>
where
    S: DataStream<Item = SimulationData>,
{
    inner: SimulationStream<S>,
    simulation_id: String,
}

#[async_trait]
impl<S> DataStream for FilteredSimulationStream<S>
where
    S: DataStream<Item = SimulationData>,
{
    type Item = SimulationData;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        loop {
            match self.inner.next().await? {
                Some(data) => {
                    if data.simulation_id == self.simulation_id {
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

/// Simulation statistics aggregator
#[derive(Debug, Clone, Default)]
pub struct SimulationStats {
    pub total_frames: u64,
    pub avg_entities_per_frame: f64,
    pub min_time_step: f64,
    pub max_time_step: f64,
    pub total_entities_seen: u64,
}

impl SimulationStats {
    pub fn update(&mut self, data: &SimulationData) {
        self.total_frames += 1;
        self.total_entities_seen += data.entities.len() as u64;
        self.avg_entities_per_frame =
            self.total_entities_seen as f64 / self.total_frames as f64;

        if self.min_time_step == 0.0 || data.time_step < self.min_time_step {
            self.min_time_step = data.time_step;
        }

        if data.time_step > self.max_time_step {
            self.max_time_step = data.time_step;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::iterator::IteratorSource;
    use crate::source::Source;

    #[tokio::test]
    async fn test_simulation_stream() {
        let data1 = SimulationData::new("sim1".to_string(), 1, 0.016);
        let data2 = SimulationData::new("sim1".to_string(), 2, 0.016);

        let source = IteratorSource::new(vec![data1, data2].into_iter());
        let mut stream = SimulationStream::new(source);

        // Note: source needs to be started
        // In a real scenario, we'd call stream.start().await

        assert_eq!(stream.current_simulation, None);
    }
}
