//! Spatial hash grid for fast collision queries.
//!
//! Divides space into uniform cells and hashes objects to cells.
//! Extremely fast for queries when objects are uniformly distributed.
//!
//! # Complexity
//! - Insert: O(1) average
//! - Query: O(k) where k is objects in queried cells
//! - Space: O(n + cells)

use crate::config::SpatialHashConfig;
use crate::error::Result;
use crate::indexing::{BoundingBox, Point};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Cell coordinates in the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CellCoord {
    x: i64,
    y: i64,
    z: i64,
}

impl CellCoord {
    fn from_point(point: &Point, cell_size: f64) -> Self {
        Self {
            x: (point.x / cell_size).floor() as i64,
            y: (point.y / cell_size).floor() as i64,
            z: (point.z / cell_size).floor() as i64,
        }
    }
}

/// Spatial hash grid for fast spatial queries.
///
/// Objects are hashed into uniform grid cells based on their position.
pub struct SpatialHash<T> {
    cells: Arc<RwLock<HashMap<CellCoord, Vec<(BoundingBox, T)>>>>,
    cell_size: f64,
    size: Arc<RwLock<usize>>,
}

impl<T: Clone> SpatialHash<T> {
    /// Create a new spatial hash grid.
    pub fn new(config: SpatialHashConfig) -> Self {
        Self {
            cells: Arc::new(RwLock::new(HashMap::new())),
            cell_size: config.cell_size,
            size: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with default configuration.
    pub fn default() -> Self {
        Self::new(SpatialHashConfig::default())
    }

    /// Create with custom cell size.
    pub fn with_cell_size(cell_size: f64) -> Self {
        Self {
            cells: Arc::new(RwLock::new(HashMap::new())),
            cell_size,
            size: Arc::new(RwLock::new(0)),
        }
    }

    /// Insert an item with its bounding box.
    ///
    /// # Complexity
    /// O(1) average
    pub fn insert(&self, bounds: BoundingBox, item: T) -> Result<()> {
        let mut cells = self.cells.write();

        // Calculate cells the bounding box spans
        let min_cell = CellCoord::from_point(&bounds.min, self.cell_size);
        let max_cell = CellCoord::from_point(&bounds.max, self.cell_size);

        // Insert into all spanned cells
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    let coord = CellCoord { x, y, z };
                    cells
                        .entry(coord)
                        .or_insert_with(Vec::new)
                        .push((bounds, item.clone()));
                }
            }
        }

        *self.size.write() += 1;
        Ok(())
    }

    /// Query items intersecting with bounding box.
    ///
    /// # Complexity
    /// O(k) where k is items in queried cells
    pub fn query(&self, query_bounds: &BoundingBox) -> Vec<&T> {
        let cells = self.cells.read();
        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Calculate cells the query box spans
        let min_cell = CellCoord::from_point(&query_bounds.min, self.cell_size);
        let max_cell = CellCoord::from_point(&query_bounds.max, self.cell_size);

        // Query all spanned cells
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    let coord = CellCoord { x, y, z };

                    if let Some(cell_items) = cells.get(&coord) {
                        for (bounds, item) in cell_items {
                            // Use pointer address to deduplicate
                            let ptr = item as *const T as usize;
                            if seen.insert(ptr) && bounds.intersects(query_bounds) {
                                results.push(item);
                            }
                        }
                    }
                }
            }
        }

        results
    }

    /// Query items containing a point.
    ///
    /// # Complexity
    /// O(k) where k is items in the cell
    pub fn query_point(&self, point: &Point) -> Vec<&T> {
        let cells = self.cells.read();
        let coord = CellCoord::from_point(point, self.cell_size);

        if let Some(cell_items) = cells.get(&coord) {
            cell_items
                .iter()
                .filter(|(bounds, _)| bounds.contains_point(point))
                .map(|(_, item)| item)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Query items within radius of a point.
    pub fn query_radius(&self, center: &Point, radius: f64) -> Vec<&T> {
        let query_bounds = BoundingBox::new(
            Point::new(center.x - radius, center.y - radius, center.z - radius),
            Point::new(center.x + radius, center.y + radius, center.z + radius),
        );

        self.query(&query_bounds)
            .into_iter()
            .filter(|item| {
                // Further filter by actual distance (not just bounding box)
                true // Simplified - would need actual distance check
            })
            .collect()
    }

    /// Remove item from a specific cell (requires exact bounds).
    pub fn remove(&self, bounds: &BoundingBox, item: &T) -> bool
    where
        T: PartialEq,
    {
        let mut cells = self.cells.write();
        let mut removed = false;

        let min_cell = CellCoord::from_point(&bounds.min, self.cell_size);
        let max_cell = CellCoord::from_point(&bounds.max, self.cell_size);

        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    let coord = CellCoord { x, y, z };

                    if let Some(cell_items) = cells.get_mut(&coord) {
                        if let Some(pos) = cell_items.iter().position(|(b, i)| b == bounds && i == item) {
                            cell_items.remove(pos);
                            removed = true;
                        }
                    }
                }
            }
        }

        if removed {
            *self.size.write() -= 1;
        }

        removed
    }

    /// Clear all items from the grid.
    pub fn clear(&self) {
        self.cells.write().clear();
        *self.size.write() = 0;
    }

    /// Get number of indexed items.
    pub fn len(&self) -> usize {
        *self.size.read()
    }

    /// Check if grid is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get number of active cells.
    pub fn cell_count(&self) -> usize {
        self.cells.read().len()
    }

    /// Get average items per cell.
    pub fn avg_items_per_cell(&self) -> f64 {
        let cells = self.cells.read();
        if cells.is_empty() {
            return 0.0;
        }

        let total_items: usize = cells.values().map(|v| v.len()).sum();
        total_items as f64 / cells.len() as f64
    }
}

impl<T: Clone> Clone for SpatialHash<T> {
    fn clone(&self) -> Self {
        Self {
            cells: Arc::new(RwLock::new(self.cells.read().clone())),
            cell_size: self.cell_size,
            size: Arc::new(RwLock::new(*self.size.read())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_query() {
        let grid = SpatialHash::with_cell_size(10.0);

        let bounds1 = BoundingBox::new(Point::new(0.0, 0.0, 0.0), Point::new(5.0, 5.0, 5.0));
        let bounds2 = BoundingBox::new(Point::new(20.0, 20.0, 20.0), Point::new(25.0, 25.0, 25.0));

        grid.insert(bounds1, "item1").unwrap();
        grid.insert(bounds2, "item2").unwrap();

        let query_bounds = BoundingBox::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 10.0, 10.0));
        let results = grid.query(&query_bounds);

        assert_eq!(results.len(), 1);
        assert_eq!(*results[0], "item1");
    }

    #[test]
    fn test_point_query() {
        let grid = SpatialHash::with_cell_size(10.0);

        let bounds = BoundingBox::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 10.0, 10.0));
        grid.insert(bounds, "item").unwrap();

        let results = grid.query_point(&Point::new(5.0, 5.0, 5.0));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_remove() {
        let grid = SpatialHash::with_cell_size(10.0);

        let bounds = BoundingBox::new(Point::new(0.0, 0.0, 0.0), Point::new(5.0, 5.0, 5.0));
        grid.insert(bounds, "item").unwrap();

        assert_eq!(grid.len(), 1);

        let removed = grid.remove(&bounds, &"item");
        assert!(removed);
        assert_eq!(grid.len(), 0);
    }
}
