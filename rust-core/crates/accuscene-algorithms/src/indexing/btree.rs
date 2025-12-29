//! Disk-friendly B+ tree implementation.
//!
//! B+ trees are optimized for disk access patterns with:
//! - All values stored in leaf nodes
//! - Internal nodes only store keys for navigation
//! - Leaf nodes linked for efficient range queries
//!
//! # Complexity
//! - Search: O(log n)
//! - Insert: O(log n)
//! - Delete: O(log n)
//! - Range query: O(log n + k) where k is result size
//! - Space: O(n)

use crate::config::BTreeConfig;
use crate::error::{AlgorithmError, Result};
use parking_lot::RwLock;
use std::cmp::Ordering;
use std::sync::Arc;

/// B+ tree node.
#[derive(Clone)]
enum Node<K: Ord + Clone, V: Clone> {
    Internal(InternalNode<K, V>),
    Leaf(LeafNode<K, V>),
}

/// Internal node (index node).
#[derive(Clone)]
struct InternalNode<K: Ord + Clone, V: Clone> {
    keys: Vec<K>,
    children: Vec<Arc<RwLock<Node<K, V>>>>,
}

/// Leaf node (data node).
#[derive(Clone)]
struct LeafNode<K: Ord + Clone, V: Clone> {
    keys: Vec<K>,
    values: Vec<V>,
    next: Option<Arc<RwLock<Node<K, V>>>>,
}

/// B+ tree for sorted key-value storage.
///
/// All data is stored in leaf nodes, making it ideal for disk-based storage
/// and range queries.
pub struct BPlusTree<K: Ord + Clone, V: Clone> {
    root: Arc<RwLock<Node<K, V>>>,
    order: usize,
    size: Arc<RwLock<usize>>,
}

impl<K: Ord + Clone, V: Clone> BPlusTree<K, V> {
    /// Create a new B+ tree with specified order.
    ///
    /// Order determines the maximum number of keys per node.
    pub fn new(config: BTreeConfig) -> Self {
        let order = config.order.max(3);
        Self {
            root: Arc::new(RwLock::new(Node::Leaf(LeafNode {
                keys: Vec::new(),
                values: Vec::new(),
                next: None,
            }))),
            order,
            size: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with default configuration.
    pub fn default() -> Self {
        Self::new(BTreeConfig::default())
    }

    /// Insert a key-value pair.
    ///
    /// # Complexity
    /// O(log n) where n is number of entries
    pub fn insert(&self, key: K, value: V) -> Result<()> {
        let mut root = self.root.write();

        // Try to insert into current root
        if let Some((new_key, new_child)) = self.insert_into_node(&mut root, key.clone(), value)? {
            // Root split - create new root
            let old_root = std::mem::replace(
                &mut *root,
                Node::Internal(InternalNode {
                    keys: vec![new_key],
                    children: vec![
                        Arc::new(RwLock::new(root.clone())),
                        Arc::new(RwLock::new(new_child)),
                    ],
                }),
            );

            // Update first child
            if let Node::Internal(ref mut internal) = *root {
                internal.children[0] = Arc::new(RwLock::new(old_root));
            }
        }

        *self.size.write() += 1;
        Ok(())
    }

    /// Insert into a node, returning split info if node splits.
    fn insert_into_node(
        &self,
        node: &mut Node<K, V>,
        key: K,
        value: V,
    ) -> Result<Option<(K, Node<K, V>)>> {
        match node {
            Node::Leaf(leaf) => {
                // Find insertion position
                let pos = leaf.keys.binary_search(&key).unwrap_or_else(|e| e);

                // Insert key-value
                leaf.keys.insert(pos, key);
                leaf.values.insert(pos, value);

                // Check if split needed
                if leaf.keys.len() > self.order {
                    let split_pos = self.order / 2;

                    // Create new leaf with right half
                    let new_leaf = LeafNode {
                        keys: leaf.keys.split_off(split_pos),
                        values: leaf.values.split_off(split_pos),
                        next: leaf.next.take(),
                    };

                    // Link leaves
                    let new_leaf_arc = Arc::new(RwLock::new(Node::Leaf(new_leaf.clone())));
                    leaf.next = Some(new_leaf_arc);

                    // Return split key (first key of new leaf)
                    Ok(Some((new_leaf.keys[0].clone(), Node::Leaf(new_leaf))))
                } else {
                    Ok(None)
                }
            }
            Node::Internal(internal) => {
                // Find child to insert into
                let child_idx = internal
                    .keys
                    .binary_search(&key)
                    .unwrap_or_else(|e| e.min(internal.children.len() - 1));

                // Insert into child
                let mut child = internal.children[child_idx].write();
                let split_result = self.insert_into_node(&mut child, key, value)?;
                drop(child);

                // Handle child split
                if let Some((split_key, new_node)) = split_result {
                    // Insert split key and new child
                    let pos = internal
                        .keys
                        .binary_search(&split_key)
                        .unwrap_or_else(|e| e);
                    internal.keys.insert(pos, split_key.clone());
                    internal
                        .children
                        .insert(pos + 1, Arc::new(RwLock::new(new_node)));

                    // Check if this node needs to split
                    if internal.keys.len() > self.order {
                        let split_pos = self.order / 2;

                        let new_internal = InternalNode {
                            keys: internal.keys.split_off(split_pos + 1),
                            children: internal.children.split_off(split_pos + 1),
                        };

                        let split_key = internal.keys.pop().unwrap();

                        Ok(Some((split_key, Node::Internal(new_internal))))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Search for a value by key.
    ///
    /// # Complexity
    /// O(log n) where n is number of entries
    pub fn search(&self, key: &K) -> Option<V> {
        let node = self.root.read();
        self.search_node(&node, key)
    }

    fn search_node(&self, node: &Node<K, V>, key: &K) -> Option<V> {
        match node {
            Node::Leaf(leaf) => {
                if let Ok(pos) = leaf.keys.binary_search(key) {
                    Some(leaf.values[pos].clone())
                } else {
                    None
                }
            }
            Node::Internal(internal) => {
                let child_idx = internal
                    .keys
                    .binary_search(key)
                    .unwrap_or_else(|e| e.min(internal.children.len() - 1));
                let child = internal.children[child_idx].read();
                self.search_node(&child, key)
            }
        }
    }

    /// Range query: get all values where min_key <= key <= max_key.
    ///
    /// # Complexity
    /// O(log n + k) where n is total entries, k is result size
    pub fn range_query(&self, min_key: &K, max_key: &K) -> Vec<V> {
        let mut results = Vec::new();
        let node = self.root.read();
        self.range_query_node(&node, min_key, max_key, &mut results);
        results
    }

    fn range_query_node(&self, node: &Node<K, V>, min_key: &K, max_key: &K, results: &mut Vec<V>) {
        match node {
            Node::Leaf(leaf) => {
                for (k, v) in leaf.keys.iter().zip(leaf.values.iter()) {
                    if k >= min_key && k <= max_key {
                        results.push(v.clone());
                    } else if k > max_key {
                        break;
                    }
                }
            }
            Node::Internal(internal) => {
                for (i, child) in internal.children.iter().enumerate() {
                    let should_search = if i == 0 {
                        true
                    } else {
                        &internal.keys[i - 1] <= max_key
                    };

                    if should_search {
                        let child_node = child.read();
                        self.range_query_node(&child_node, min_key, max_key, results);
                    }
                }
            }
        }
    }

    /// Get number of entries.
    pub fn len(&self) -> usize {
        *self.size.read()
    }

    /// Check if tree is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get all keys in sorted order.
    pub fn keys(&self) -> Vec<K> {
        let mut keys = Vec::new();
        let node = self.root.read();
        self.collect_keys(&node, &mut keys);
        keys
    }

    fn collect_keys(&self, node: &Node<K, V>, keys: &mut Vec<K>) {
        match node {
            Node::Leaf(leaf) => {
                keys.extend(leaf.keys.iter().cloned());
            }
            Node::Internal(internal) => {
                for child in &internal.children {
                    let child_node = child.read();
                    self.collect_keys(&child_node, keys);
                }
            }
        }
    }
}

impl<K: Ord + Clone, V: Clone> Clone for BPlusTree<K, V> {
    fn clone(&self) -> Self {
        Self {
            root: Arc::new(RwLock::new(self.root.read().clone())),
            order: self.order,
            size: Arc::new(RwLock::new(*self.size.read())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_search() {
        let tree = BPlusTree::default();

        tree.insert(5, "five").unwrap();
        tree.insert(3, "three").unwrap();
        tree.insert(7, "seven").unwrap();
        tree.insert(1, "one").unwrap();

        assert_eq!(tree.search(&5), Some("five"));
        assert_eq!(tree.search(&3), Some("three"));
        assert_eq!(tree.search(&7), Some("seven"));
        assert_eq!(tree.search(&1), Some("one"));
        assert_eq!(tree.search(&10), None);
    }

    #[test]
    fn test_range_query() {
        let tree = BPlusTree::default();

        for i in 0..100 {
            tree.insert(i, i * 2).unwrap();
        }

        let results = tree.range_query(&10, &20);
        assert_eq!(results.len(), 11);
    }

    #[test]
    fn test_many_inserts() {
        let tree = BPlusTree::default();

        for i in 0..1000 {
            tree.insert(i, i).unwrap();
        }

        assert_eq!(tree.len(), 1000);

        for i in 0..1000 {
            assert_eq!(tree.search(&i), Some(i));
        }
    }
}
