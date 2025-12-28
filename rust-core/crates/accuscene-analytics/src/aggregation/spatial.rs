//! Spatial aggregations for location-based analytics

use super::{AggregationOp, AggregationResult, Aggregator, MeanAggregator, SumAggregator};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A point in 2D space (latitude, longitude)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpatialPoint {
    pub latitude: f64,
    pub longitude: f64,
}

impl SpatialPoint {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self { latitude, longitude }
    }

    /// Calculate distance to another point in meters (Haversine formula)
    pub fn distance_to(&self, other: &SpatialPoint) -> f64 {
        const EARTH_RADIUS: f64 = 6371000.0; // meters

        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let delta_lat = (other.latitude - self.latitude).to_radians();
        let delta_lon = (other.longitude - self.longitude).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();

        EARTH_RADIUS * c
    }
}

/// Grid cell identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridCell {
    pub x: i32,
    pub y: i32,
}

impl GridCell {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to_string(&self) -> String {
        format!("{}_{}", self.x, self.y)
    }
}

/// Spatial grid for discretizing space into cells
#[derive(Debug, Clone)]
pub struct SpatialGrid {
    /// Cell size in meters
    cell_size: f64,
    /// Origin point (southwest corner)
    origin: SpatialPoint,
}

impl SpatialGrid {
    pub fn new(origin: SpatialPoint, cell_size: f64) -> Self {
        Self { origin, cell_size }
    }

    /// Convert a spatial point to a grid cell
    pub fn point_to_cell(&self, point: &SpatialPoint) -> GridCell {
        // Approximate conversion (works for small areas)
        const METERS_PER_DEGREE_LAT: f64 = 111320.0;
        let meters_per_degree_lon = METERS_PER_DEGREE_LAT * self.origin.latitude.to_radians().cos();

        let lat_offset = (point.latitude - self.origin.latitude) * METERS_PER_DEGREE_LAT;
        let lon_offset = (point.longitude - self.origin.longitude) * meters_per_degree_lon;

        GridCell {
            x: (lon_offset / self.cell_size).floor() as i32,
            y: (lat_offset / self.cell_size).floor() as i32,
        }
    }

    /// Convert a grid cell to its center point
    pub fn cell_to_point(&self, cell: &GridCell) -> SpatialPoint {
        const METERS_PER_DEGREE_LAT: f64 = 111320.0;
        let meters_per_degree_lon = METERS_PER_DEGREE_LAT * self.origin.latitude.to_radians().cos();

        let lat_offset = (cell.y as f64 + 0.5) * self.cell_size / METERS_PER_DEGREE_LAT;
        let lon_offset = (cell.x as f64 + 0.5) * self.cell_size / meters_per_degree_lon;

        SpatialPoint {
            latitude: self.origin.latitude + lat_offset,
            longitude: self.origin.longitude + lon_offset,
        }
    }

    /// Get neighboring cells (8-connected)
    pub fn neighbors(&self, cell: &GridCell) -> Vec<GridCell> {
        vec![
            GridCell::new(cell.x - 1, cell.y - 1),
            GridCell::new(cell.x, cell.y - 1),
            GridCell::new(cell.x + 1, cell.y - 1),
            GridCell::new(cell.x - 1, cell.y),
            GridCell::new(cell.x + 1, cell.y),
            GridCell::new(cell.x - 1, cell.y + 1),
            GridCell::new(cell.x, cell.y + 1),
            GridCell::new(cell.x + 1, cell.y + 1),
        ]
    }
}

/// Spatial aggregator for location-based data
pub struct SpatialAggregator {
    grid: SpatialGrid,
    operation: AggregationOp,
    cells: Arc<DashMap<GridCell, Box<dyn Aggregator + Send + Sync>>>,
}

impl SpatialAggregator {
    pub fn new(grid: SpatialGrid, operation: AggregationOp) -> Self {
        Self {
            grid,
            operation,
            cells: Arc::new(DashMap::new()),
        }
    }

    /// Add a value at a specific location
    pub fn add(&self, point: SpatialPoint, value: f64) {
        let cell = self.grid.point_to_cell(&point);

        self.cells
            .entry(cell)
            .or_insert_with(|| self.create_aggregator())
            .add(value);
    }

    /// Get aggregation results for all cells
    pub fn results(&self) -> Vec<(GridCell, SpatialPoint, AggregationResult)> {
        self.cells
            .iter()
            .map(|entry| {
                let cell = *entry.key();
                let agg = entry.value();
                let point = self.grid.cell_to_point(&cell);
                let result = AggregationResult::new(self.operation, agg.result(), agg.count());
                (cell, point, result)
            })
            .collect()
    }

    /// Get result for a specific cell
    pub fn result_for_cell(&self, cell: GridCell) -> Option<AggregationResult> {
        self.cells.get(&cell).map(|agg| {
            AggregationResult::new(self.operation, agg.result(), agg.count())
        })
    }

    /// Get result for a point (maps to cell)
    pub fn result_for_point(&self, point: SpatialPoint) -> Option<AggregationResult> {
        let cell = self.grid.point_to_cell(&point);
        self.result_for_cell(cell)
    }

    /// Get all cells within a radius of a point
    pub fn results_in_radius(
        &self,
        center: SpatialPoint,
        radius_meters: f64,
    ) -> Vec<(GridCell, SpatialPoint, AggregationResult)> {
        self.results()
            .into_iter()
            .filter(|(_, point, _)| center.distance_to(point) <= radius_meters)
            .collect()
    }

    /// Get hotspots (cells with values above threshold)
    pub fn hotspots(&self, threshold: f64) -> Vec<(GridCell, SpatialPoint, AggregationResult)> {
        self.results()
            .into_iter()
            .filter(|(_, _, result)| result.value >= threshold)
            .collect()
    }

    /// Clear all cells
    pub fn clear(&self) {
        self.cells.clear();
    }

    /// Get the number of cells
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    fn create_aggregator(&self) -> Box<dyn Aggregator + Send + Sync> {
        match self.operation {
            AggregationOp::Sum | AggregationOp::Count => Box::new(SumAggregator::default()),
            AggregationOp::Mean => Box::new(MeanAggregator::default()),
            _ => Box::new(SumAggregator::default()),
        }
    }
}

/// Cluster-based spatial aggregator using density-based clustering
pub struct SpatialClusterAggregator {
    epsilon: f64, // Maximum distance for clustering
    min_points: usize, // Minimum points to form a cluster
    points: Arc<DashMap<uuid::Uuid, (SpatialPoint, f64)>>,
}

impl SpatialClusterAggregator {
    pub fn new(epsilon: f64, min_points: usize) -> Self {
        Self {
            epsilon,
            min_points,
            points: Arc::new(DashMap::new()),
        }
    }

    /// Add a point with a value
    pub fn add(&self, point: SpatialPoint, value: f64) {
        let id = uuid::Uuid::new_v4();
        self.points.insert(id, (point, value));
    }

    /// Find clusters (simplified DBSCAN)
    pub fn clusters(&self) -> Vec<Cluster> {
        let points: Vec<_> = self.points.iter().map(|e| (*e.key(), e.value().0, e.value().1)).collect();

        let mut visited = std::collections::HashSet::new();
        let mut clusters = Vec::new();

        for (id, point, value) in &points {
            if visited.contains(id) {
                continue;
            }

            let neighbors: Vec<_> = points
                .iter()
                .filter(|(_, p, _)| point.distance_to(p) <= self.epsilon)
                .collect();

            if neighbors.len() >= self.min_points {
                let mut cluster = Cluster {
                    center: *point,
                    points: Vec::new(),
                    total_value: 0.0,
                };

                for (nid, npoint, nvalue) in neighbors {
                    visited.insert(*nid);
                    cluster.points.push(*npoint);
                    cluster.total_value += nvalue;
                }

                // Update center to centroid
                let lat_sum: f64 = cluster.points.iter().map(|p| p.latitude).sum();
                let lon_sum: f64 = cluster.points.iter().map(|p| p.longitude).sum();
                cluster.center = SpatialPoint::new(
                    lat_sum / cluster.points.len() as f64,
                    lon_sum / cluster.points.len() as f64,
                );

                clusters.push(cluster);
            }
        }

        clusters
    }

    /// Clear all points
    pub fn clear(&self) {
        self.points.clear();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub center: SpatialPoint,
    pub points: Vec<SpatialPoint>,
    pub total_value: f64,
}

impl Cluster {
    pub fn size(&self) -> usize {
        self.points.len()
    }

    pub fn mean_value(&self) -> f64 {
        if self.points.is_empty() {
            0.0
        } else {
            self.total_value / self.points.len() as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_point_distance() {
        let p1 = SpatialPoint::new(40.7128, -74.0060); // New York
        let p2 = SpatialPoint::new(34.0522, -118.2437); // Los Angeles

        let distance = p1.distance_to(&p2);
        assert!(distance > 3_900_000.0); // ~3900 km
        assert!(distance < 4_000_000.0);
    }

    #[test]
    fn test_spatial_grid() {
        let origin = SpatialPoint::new(0.0, 0.0);
        let grid = SpatialGrid::new(origin, 1000.0); // 1km cells

        let point = SpatialPoint::new(0.01, 0.01);
        let cell = grid.point_to_cell(&point);

        assert!(cell.x >= 0);
        assert!(cell.y >= 0);
    }

    #[test]
    fn test_spatial_aggregator() {
        let origin = SpatialPoint::new(0.0, 0.0);
        let grid = SpatialGrid::new(origin, 1000.0);
        let agg = SpatialAggregator::new(grid, AggregationOp::Sum);

        let point = SpatialPoint::new(0.001, 0.001);
        agg.add(point, 10.0);
        agg.add(point, 20.0);

        let results = agg.results();
        assert!(!results.is_empty());
    }
}
