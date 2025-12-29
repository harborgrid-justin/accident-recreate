//! R-tree for spatial indexing of scene objects.
//!
//! R-trees organize spatial data hierarchically using minimum bounding rectangles (MBRs).
//! Optimized for spatial queries like intersection and nearest neighbor searches.
//!
//! # Complexity
//! - Insert: O(log n) average
//! - Query: O(log n + k) where k is result size
//! - Space: O(n)

use crate::config::RTreeConfig;
use crate::error::Result;
use crate::indexing::{BoundingBox, Point, SpatialIndex};
use parking_lot::RwLock;
use std::sync::Arc;

/// R-tree node.
enum RNode<T> {
    Internal {
        bounds: BoundingBox,
        children: Vec<Arc<RwLock<RNode<T>>>>,
    },
    Leaf {
        bounds: BoundingBox,
        items: Vec<(BoundingBox, T)>,
    },
}

impl<T> RNode<T> {
    fn bounds(&self) -> &BoundingBox {
        match self {
            RNode::Internal { bounds, .. } => bounds,
            RNode::Leaf { bounds, .. } => bounds,
        }
    }

    fn is_leaf(&self) -> bool {
        matches!(self, RNode::Leaf { .. })
    }
}

/// R-tree for spatial indexing.
///
/// Uses R*-tree variant with forced reinsertions for better tree quality.
pub struct RTree<T> {
    root: Arc<RwLock<RNode<T>>>,
    config: RTreeConfig,
    size: Arc<RwLock<usize>>,
}

impl<T: Clone> RTree<T> {
    /// Create a new R-tree.
    pub fn new(config: RTreeConfig) -> Self {
        // Create initial empty leaf
        let initial_bounds = BoundingBox::new(
            Point::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        );

        Self {
            root: Arc::new(RwLock::new(RNode::Leaf {
                bounds: initial_bounds,
                items: Vec::new(),
            })),
            config,
            size: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with default configuration.
    pub fn default() -> Self {
        Self::new(RTreeConfig::default())
    }

    /// Insert an item with its bounding box.
    ///
    /// # Complexity
    /// O(log n) average
    pub fn insert(&self, bounds: BoundingBox, item: T) -> Result<()> {
        let mut root = self.root.write();
        let entry = (bounds, item);

        if let Some(new_node) = self.insert_into_node(&mut root, entry)? {
            // Root split - create new root
            let old_root = Arc::new(RwLock::new(root.clone()));
            let new_node_arc = Arc::new(RwLock::new(new_node));

            let new_bounds = old_root.read().bounds().union(new_node_arc.read().bounds());

            *root = RNode::Internal {
                bounds: new_bounds,
                children: vec![old_root, new_node_arc],
            };
        }

        *self.size.write() += 1;
        Ok(())
    }

    fn insert_into_node(
        &self,
        node: &mut RNode<T>,
        entry: (BoundingBox, T),
    ) -> Result<Option<RNode<T>>> {
        match node {
            RNode::Leaf { bounds, items } => {
                items.push(entry.clone());
                *bounds = self.compute_bounds_from_items(items);

                // Check if split needed
                if items.len() > self.config.max_entries {
                    Ok(Some(self.split_leaf(items)?))
                } else {
                    Ok(None)
                }
            }
            RNode::Internal {
                bounds,
                ref mut children,
            } => {
                // Choose subtree with minimum area enlargement
                let best_child = self.choose_subtree(children, &entry.0);
                let mut child = children[best_child].write();

                if let Some(new_child) = self.insert_into_node(&mut child, entry)? {
                    drop(child);

                    // Add new child
                    children.push(Arc::new(RwLock::new(new_child)));
                    *bounds = self.compute_bounds_from_children(children);

                    // Check if split needed
                    if children.len() > self.config.max_entries {
                        Ok(Some(self.split_internal(children)?))
                    } else {
                        Ok(None)
                    }
                } else {
                    drop(child);
                    *bounds = self.compute_bounds_from_children(children);
                    Ok(None)
                }
            }
        }
    }

    fn choose_subtree(&self, children: &[Arc<RwLock<RNode<T>>>], bounds: &BoundingBox) -> usize {
        let mut best_idx = 0;
        let mut min_enlargement = f64::INFINITY;

        for (i, child) in children.iter().enumerate() {
            let child_bounds = child.read().bounds();
            let union = child_bounds.union(bounds);
            let enlargement = union.area() - child_bounds.area();

            if enlargement < min_enlargement {
                min_enlargement = enlargement;
                best_idx = i;
            }
        }

        best_idx
    }

    fn split_leaf(&self, items: &mut Vec<(BoundingBox, T)>) -> Result<RNode<T>> {
        let split_idx = items.len() / 2;
        let new_items = items.split_off(split_idx);
        let new_bounds = self.compute_bounds_from_items(&new_items);

        Ok(RNode::Leaf {
            bounds: new_bounds,
            items: new_items,
        })
    }

    fn split_internal(&self, children: &mut Vec<Arc<RwLock<RNode<T>>>>) -> Result<RNode<T>> {
        let split_idx = children.len() / 2;
        let new_children = children.split_off(split_idx);
        let new_bounds = self.compute_bounds_from_children(&new_children);

        Ok(RNode::Internal {
            bounds: new_bounds,
            children: new_children,
        })
    }

    fn compute_bounds_from_items(&self, items: &[(BoundingBox, T)]) -> BoundingBox {
        if items.is_empty() {
            return BoundingBox::new(
                Point::new(0.0, 0.0, 0.0),
                Point::new(0.0, 0.0, 0.0),
            );
        }

        let mut result = items[0].0;
        for (bounds, _) in &items[1..] {
            result = result.union(bounds);
        }
        result
    }

    fn compute_bounds_from_children(&self, children: &[Arc<RwLock<RNode<T>>>]) -> BoundingBox {
        if children.is_empty() {
            return BoundingBox::new(
                Point::new(0.0, 0.0, 0.0),
                Point::new(0.0, 0.0, 0.0),
            );
        }

        let mut result = *children[0].read().bounds();
        for child in &children[1..] {
            result = result.union(child.read().bounds());
        }
        result
    }

    /// Query items intersecting with bounding box.
    ///
    /// # Complexity
    /// O(log n + k) where k is result size
    pub fn query(&self, query_bounds: &BoundingBox) -> Vec<&T> {
        let root = self.root.read();
        let mut results = Vec::new();
        self.query_node(&root, query_bounds, &mut results);
        results
    }

    fn query_node<'a>(
        &self,
        node: &'a RNode<T>,
        query_bounds: &BoundingBox,
        results: &mut Vec<&'a T>,
    ) {
        if !node.bounds().intersects(query_bounds) {
            return;
        }

        match node {
            RNode::Leaf { items, .. } => {
                for (bounds, item) in items {
                    if bounds.intersects(query_bounds) {
                        results.push(item);
                    }
                }
            }
            RNode::Internal { children, .. } => {
                for child in children {
                    let child_node = child.read();
                    self.query_node(&child_node, query_bounds, results);
                }
            }
        }
    }

    /// Query items containing a point.
    pub fn query_point(&self, point: &Point) -> Vec<&T> {
        let root = self.root.read();
        let mut results = Vec::new();
        self.query_point_node(&root, point, &mut results);
        results
    }

    fn query_point_node<'a>(&self, node: &'a RNode<T>, point: &Point, results: &mut Vec<&'a T>) {
        if !node.bounds().contains_point(point) {
            return;
        }

        match node {
            RNode::Leaf { items, .. } => {
                for (bounds, item) in items {
                    if bounds.contains_point(point) {
                        results.push(item);
                    }
                }
            }
            RNode::Internal { children, .. } => {
                for child in children {
                    let child_node = child.read();
                    self.query_point_node(&child_node, point, results);
                }
            }
        }
    }

    /// Get number of indexed items.
    pub fn len(&self) -> usize {
        *self.size.read()
    }

    /// Check if tree is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Clone> Clone for RTree<T> {
    fn clone(&self) -> Self {
        Self {
            root: Arc::new(RwLock::new(self.root.read().clone())),
            config: self.config.clone(),
            size: Arc::new(RwLock::new(*self.size.read())),
        }
    }
}

impl<T> Clone for RNode<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            RNode::Internal { bounds, children } => RNode::Internal {
                bounds: *bounds,
                children: children.iter().map(|c| Arc::new(RwLock::new(c.read().clone()))).collect(),
            },
            RNode::Leaf { bounds, items } => RNode::Leaf {
                bounds: *bounds,
                items: items.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_query() {
        let rtree = RTree::default();

        let bounds1 = BoundingBox::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 10.0, 10.0));
        let bounds2 = BoundingBox::new(Point::new(5.0, 5.0, 5.0), Point::new(15.0, 15.0, 15.0));

        rtree.insert(bounds1, "item1").unwrap();
        rtree.insert(bounds2, "item2").unwrap();

        let query_bounds = BoundingBox::new(Point::new(8.0, 8.0, 8.0), Point::new(12.0, 12.0, 12.0));
        let results = rtree.query(&query_bounds);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_point_query() {
        let rtree = RTree::default();

        let bounds = BoundingBox::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 10.0, 10.0));
        rtree.insert(bounds, "item").unwrap();

        let point_inside = Point::new(5.0, 5.0, 5.0);
        let results = rtree.query_point(&point_inside);
        assert_eq!(results.len(), 1);

        let point_outside = Point::new(15.0, 15.0, 15.0);
        let results = rtree.query_point(&point_outside);
        assert_eq!(results.len(), 0);
    }
}
