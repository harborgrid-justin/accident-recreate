use crate::error::{OfflineError, Result};
use crate::versioning::Version;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;

/// Priority levels for operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Type of sync operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Create,
    Update,
    Delete,
    Patch,
}

/// A pending sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperation {
    /// Unique operation ID
    pub id: String,

    /// Entity ID being modified
    pub entity_id: String,

    /// Entity type (e.g., "accident", "vehicle", "user")
    pub entity_type: String,

    /// Operation type
    pub operation_type: OperationType,

    /// Operation data (JSON payload)
    pub data: serde_json::Value,

    /// Version information
    pub version: Version,

    /// Priority
    pub priority: Priority,

    /// Timestamp when operation was queued
    pub queued_at: chrono::DateTime<chrono::Utc>,

    /// Number of retry attempts
    pub retry_count: u32,

    /// Last error if any
    pub last_error: Option<String>,

    /// Dependencies (operation IDs that must complete first)
    pub dependencies: Vec<String>,

    /// Tags for categorization
    pub tags: Vec<String>,
}

impl SyncOperation {
    /// Create a new sync operation
    pub fn new(
        entity_id: String,
        entity_type: String,
        operation_type: OperationType,
        data: serde_json::Value,
        version: Version,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            entity_id,
            entity_type,
            operation_type,
            data,
            version,
            priority: Priority::Normal,
            queued_at: chrono::Utc::now(),
            retry_count: 0,
            last_error: None,
            dependencies: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Add dependency
    pub fn with_dependency(mut self, operation_id: String) -> Self {
        self.dependencies.push(operation_id);
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Set error
    pub fn set_error(&mut self, error: String) {
        self.last_error = Some(error);
    }
}

/// Wrapper for priority queue ordering
#[derive(Debug, Clone)]
struct PriorityOperation {
    operation: SyncOperation,
}

impl PartialEq for PriorityOperation {
    fn eq(&self, other: &Self) -> bool {
        self.operation.id == other.operation.id
    }
}

impl Eq for PriorityOperation {}

impl PartialOrd for PriorityOperation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityOperation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority operations come first
        self.operation.priority
            .cmp(&other.operation.priority)
            .then_with(|| {
                // Earlier operations come first if priority is equal
                other.operation.queued_at.cmp(&self.operation.queued_at)
            })
    }
}

/// Thread-safe operation queue with priority support
pub struct OperationQueue {
    /// Priority queue for operations
    queue: Arc<RwLock<BinaryHeap<PriorityOperation>>>,

    /// Quick lookup by operation ID
    operations: Arc<RwLock<HashMap<String, SyncOperation>>>,

    /// Completed operation IDs (for dependency tracking)
    completed: Arc<RwLock<HashMap<String, chrono::DateTime<chrono::Utc>>>>,

    /// Maximum queue size
    max_size: usize,
}

impl OperationQueue {
    /// Create a new operation queue
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            operations: Arc::new(RwLock::new(HashMap::new())),
            completed: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }

    /// Enqueue an operation
    pub fn enqueue(&self, operation: SyncOperation) -> Result<()> {
        let mut queue = self.queue.write();
        let mut operations = self.operations.write();

        if operations.len() >= self.max_size {
            return Err(OfflineError::QueueFull);
        }

        let op_id = operation.id.clone();
        operations.insert(op_id, operation.clone());
        queue.push(PriorityOperation { operation });

        Ok(())
    }

    /// Dequeue the highest priority operation that's ready to execute
    pub fn dequeue(&self) -> Option<SyncOperation> {
        let mut queue = self.queue.write();
        let mut operations = self.operations.write();
        let completed = self.completed.read();

        // Find the first operation with all dependencies satisfied
        let mut temp_queue = BinaryHeap::new();
        let mut result = None;

        while let Some(priority_op) = queue.pop() {
            let op = &priority_op.operation;

            // Check if all dependencies are completed
            let deps_satisfied = op.dependencies.iter()
                .all(|dep_id| completed.contains_key(dep_id));

            if deps_satisfied {
                result = Some(op.clone());
                operations.remove(&op.id);
                break;
            } else {
                temp_queue.push(priority_op);
            }
        }

        // Put back operations that weren't dequeued
        while let Some(op) = temp_queue.pop() {
            queue.push(op);
        }

        result
    }

    /// Peek at the next operation without removing it
    pub fn peek(&self) -> Option<SyncOperation> {
        let queue = self.queue.read();
        queue.peek().map(|po| po.operation.clone())
    }

    /// Mark an operation as completed
    pub fn mark_completed(&self, operation_id: &str) {
        let mut completed = self.completed.write();
        completed.insert(operation_id.to_string(), chrono::Utc::now());

        // Clean up old completed operations (keep last 1000)
        if completed.len() > 1000 {
            let mut entries: Vec<_> = completed.iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            entries.sort_by_key(|(_, timestamp)| *timestamp);
            entries.truncate(500);

            *completed = entries.into_iter().collect();
        }
    }

    /// Get operation by ID
    pub fn get(&self, operation_id: &str) -> Option<SyncOperation> {
        let operations = self.operations.read();
        operations.get(operation_id).cloned()
    }

    /// Update an existing operation
    pub fn update(&self, operation: SyncOperation) -> Result<()> {
        let mut operations = self.operations.write();

        if !operations.contains_key(&operation.id) {
            return Err(OfflineError::InvalidOperation(
                format!("Operation {} not found", operation.id)
            ));
        }

        operations.insert(operation.id.clone(), operation);
        Ok(())
    }

    /// Remove an operation
    pub fn remove(&self, operation_id: &str) -> Option<SyncOperation> {
        let mut operations = self.operations.write();
        operations.remove(operation_id)
    }

    /// Get queue size
    pub fn len(&self) -> usize {
        let operations = self.operations.read();
        operations.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get all pending operations
    pub fn pending_operations(&self) -> Vec<SyncOperation> {
        let operations = self.operations.read();
        operations.values().cloned().collect()
    }

    /// Get operations by entity
    pub fn operations_for_entity(&self, entity_id: &str) -> Vec<SyncOperation> {
        let operations = self.operations.read();
        operations.values()
            .filter(|op| op.entity_id == entity_id)
            .cloned()
            .collect()
    }

    /// Get operations by tag
    pub fn operations_with_tag(&self, tag: &str) -> Vec<SyncOperation> {
        let operations = self.operations.read();
        operations.values()
            .filter(|op| op.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }

    /// Clear all operations
    pub fn clear(&self) {
        let mut queue = self.queue.write();
        let mut operations = self.operations.write();

        queue.clear();
        operations.clear();
    }

    /// Get queue statistics
    pub fn stats(&self) -> QueueStats {
        let operations = self.operations.read();
        let completed = self.completed.read();

        let mut by_priority = HashMap::new();
        let mut by_type = HashMap::new();

        for op in operations.values() {
            *by_priority.entry(op.priority).or_insert(0) += 1;
            *by_type.entry(op.operation_type.clone()).or_insert(0) += 1;
        }

        QueueStats {
            pending: operations.len(),
            completed: completed.len(),
            by_priority,
            by_type,
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize)]
pub struct QueueStats {
    pub pending: usize,
    pub completed: usize,
    pub by_priority: HashMap<Priority, usize>,
    pub by_type: HashMap<OperationType, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::versioning::VectorClock;

    fn create_test_operation() -> SyncOperation {
        let version = Version {
            clock: VectorClock::new(),
            node_id: "test-node".to_string(),
            timestamp: chrono::Utc::now(),
            content_hash: "test-hash".to_string(),
        };

        SyncOperation::new(
            "entity-1".to_string(),
            "accident".to_string(),
            OperationType::Create,
            serde_json::json!({"test": "data"}),
            version,
        )
    }

    #[test]
    fn test_enqueue_dequeue() {
        let queue = OperationQueue::new(100);
        let op = create_test_operation();
        let op_id = op.id.clone();

        queue.enqueue(op).unwrap();
        assert_eq!(queue.len(), 1);

        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.id, op_id);
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_priority_ordering() {
        let queue = OperationQueue::new(100);

        let mut low_op = create_test_operation();
        low_op.priority = Priority::Low;

        let mut high_op = create_test_operation();
        high_op.priority = Priority::High;

        queue.enqueue(low_op).unwrap();
        queue.enqueue(high_op.clone()).unwrap();

        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.id, high_op.id);
    }

    #[test]
    fn test_dependencies() {
        let queue = OperationQueue::new(100);

        let op1 = create_test_operation();
        let op1_id = op1.id.clone();

        let op2 = create_test_operation().with_dependency(op1_id.clone());

        queue.enqueue(op1).unwrap();
        queue.enqueue(op2.clone()).unwrap();

        // Dequeue op1 first
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.id, op1_id);

        // op2 should not be dequeued yet (dependency not marked complete)
        assert!(queue.dequeue().is_none());

        // Mark op1 as completed
        queue.mark_completed(&op1_id);

        // Now op2 should be dequeued
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.id, op2.id);
    }
}
