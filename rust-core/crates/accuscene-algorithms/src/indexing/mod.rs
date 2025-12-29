//! Indexing data structures for efficient scene data queries.
//!
//! This module provides various indexing structures:
//! - B+ tree for disk-friendly sorted data
//! - R-tree for spatial indexing
//! - Spatial hash grid for fast collision queries
//! - Bloom filter for existence checks
//! - Cuckoo filter for space-efficient lookups

pub mod bloom;
pub mod btree;
pub mod cuckoo;
pub mod rtree;
pub mod spatial_hash;

pub use bloom::BloomFilter;
pub use btree::BPlusTree;
pub use cuckoo::CuckooFilter;
pub use rtree::RTree;
pub use spatial_hash::SpatialHash;

/// Common traits for indexing structures.
pub trait Index<K, V> {
    /// Insert a key-value pair.
    fn insert(&mut self, key: K, value: V) -> crate::error::Result<()>;

    /// Search for a value by key.
    fn search(&self, key: &K) -> Option<&V>;

    /// Remove a key-value pair.
    fn remove(&mut self, key: &K) -> Option<V>;

    /// Check if key exists.
    fn contains(&self, key: &K) -> bool {
        self.search(key).is_some()
    }

    /// Get number of entries.
    fn len(&self) -> usize;

    /// Check if index is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Spatial indexing trait for geometric queries.
pub trait SpatialIndex<T> {
    /// Insert an item with bounding box.
    fn insert(&mut self, bounds: BoundingBox, item: T) -> crate::error::Result<()>;

    /// Query items intersecting with bounding box.
    fn query(&self, bounds: &BoundingBox) -> Vec<&T>;

    /// Query items containing a point.
    fn query_point(&self, point: &Point) -> Vec<&T>;

    /// Remove an item.
    fn remove(&mut self, bounds: &BoundingBox, item: &T) -> bool
    where
        T: PartialEq;

    /// Get number of indexed items.
    fn len(&self) -> usize;

    /// Check if index is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// 2D/3D point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn new_2d(x: f64, y: f64) -> Self {
        Self { x, y, z: 0.0 }
    }

    pub fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min: Point,
    pub max: Point,
}

impl BoundingBox {
    pub fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }

    pub fn from_points(points: &[Point]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min = points[0];
        let mut max = points[0];

        for point in &points[1..] {
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            min.z = min.z.min(point.z);
            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
            max.z = max.z.max(point.z);
        }

        Some(Self { min, max })
    }

    pub fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    pub fn area(&self) -> f64 {
        let dx = self.max.x - self.min.x;
        let dy = self.max.y - self.min.y;
        let dz = self.max.z - self.min.z;
        2.0 * (dx * dy + dy * dz + dz * dx)
    }

    pub fn volume(&self) -> f64 {
        let dx = self.max.x - self.min.x;
        let dy = self.max.y - self.min.y;
        let dz = self.max.z - self.min.z;
        dx * dy * dz
    }

    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min: Point {
                x: self.min.x.min(other.min.x),
                y: self.min.y.min(other.min.y),
                z: self.min.z.min(other.min.z),
            },
            max: Point {
                x: self.max.x.max(other.max.x),
                y: self.max.y.max(other.max.y),
                z: self.max.z.max(other.max.z),
            },
        }
    }

    pub fn center(&self) -> Point {
        Point {
            x: (self.min.x + self.max.x) / 2.0,
            y: (self.min.y + self.max.y) / 2.0,
            z: (self.min.z + self.max.z) / 2.0,
        }
    }
}
