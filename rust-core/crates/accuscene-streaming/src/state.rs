//! State management for stateful stream processing.

use crate::error::{Result, StreamingError};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait for state backends
#[async_trait::async_trait]
pub trait StateBackend: Send + Sync {
    /// Get a value from state
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Put a value into state
    async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;

    /// Delete a value from state
    async fn delete(&self, key: &[u8]) -> Result<()>;

    /// Clear all state
    async fn clear(&self) -> Result<()>;

    /// Create a snapshot of the state
    async fn snapshot(&self) -> Result<HashMap<Vec<u8>, Vec<u8>>>;

    /// Restore state from a snapshot
    async fn restore(&self, snapshot: HashMap<Vec<u8>, Vec<u8>>) -> Result<()>;
}

/// In-memory state backend using DashMap for concurrent access
pub struct MemoryStateBackend {
    state: Arc<DashMap<Vec<u8>, Vec<u8>>>,
}

impl MemoryStateBackend {
    /// Create a new memory state backend
    pub fn new() -> Self {
        Self {
            state: Arc::new(DashMap::new()),
        }
    }
}

impl Default for MemoryStateBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl StateBackend for MemoryStateBackend {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.state.get(key).map(|v| v.value().clone()))
    }

    async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.state.insert(key, value);
        Ok(())
    }

    async fn delete(&self, key: &[u8]) -> Result<()> {
        self.state.remove(key);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.state.clear();
        Ok(())
    }

    async fn snapshot(&self) -> Result<HashMap<Vec<u8>, Vec<u8>>> {
        let snapshot = self
            .state
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        Ok(snapshot)
    }

    async fn restore(&self, snapshot: HashMap<Vec<u8>, Vec<u8>>) -> Result<()> {
        self.clear().await?;
        for (key, value) in snapshot {
            self.state.insert(key, value);
        }
        Ok(())
    }
}

/// Value state that stores a single value per key
pub struct ValueState<K, V> {
    backend: Arc<dyn StateBackend>,
    namespace: String,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> ValueState<K, V>
where
    K: Serialize + for<'de> Deserialize<'de>,
    V: Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new value state
    pub fn new(backend: Arc<dyn StateBackend>, namespace: impl Into<String>) -> Self {
        Self {
            backend,
            namespace: namespace.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    fn encode_key(&self, key: &K) -> Result<Vec<u8>> {
        let key_bytes = serde_json::to_vec(key).map_err(|e| {
            StreamingError::StateBackend(format!("Failed to encode key: {}", e))
        })?;

        let mut full_key = self.namespace.as_bytes().to_vec();
        full_key.push(b':');
        full_key.extend_from_slice(&key_bytes);

        Ok(full_key)
    }

    /// Get value for a key
    pub async fn get(&self, key: &K) -> Result<Option<V>> {
        let encoded_key = self.encode_key(key)?;
        match self.backend.get(&encoded_key).await? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes).map_err(|e| {
                    StreamingError::StateBackend(format!("Failed to decode value: {}", e))
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Put value for a key
    pub async fn put(&self, key: &K, value: &V) -> Result<()> {
        let encoded_key = self.encode_key(key)?;
        let encoded_value = serde_json::to_vec(value).map_err(|e| {
            StreamingError::StateBackend(format!("Failed to encode value: {}", e))
        })?;

        self.backend.put(encoded_key, encoded_value).await
    }

    /// Delete value for a key
    pub async fn delete(&self, key: &K) -> Result<()> {
        let encoded_key = self.encode_key(key)?;
        self.backend.delete(&encoded_key).await
    }

    /// Update value using a function
    pub async fn update<F>(&self, key: &K, f: F) -> Result<()>
    where
        F: FnOnce(Option<V>) -> Option<V>,
    {
        let current = self.get(key).await?;
        match f(current) {
            Some(new_value) => self.put(key, &new_value).await,
            None => self.delete(key).await,
        }
    }
}

/// List state that stores a list of values per key
pub struct ListState<K, V> {
    backend: Arc<dyn StateBackend>,
    namespace: String,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> ListState<K, V>
where
    K: Serialize + for<'de> Deserialize<'de>,
    V: Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new list state
    pub fn new(backend: Arc<dyn StateBackend>, namespace: impl Into<String>) -> Self {
        Self {
            backend,
            namespace: namespace.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    fn encode_key(&self, key: &K) -> Result<Vec<u8>> {
        let key_bytes = serde_json::to_vec(key).map_err(|e| {
            StreamingError::StateBackend(format!("Failed to encode key: {}", e))
        })?;

        let mut full_key = self.namespace.as_bytes().to_vec();
        full_key.push(b':');
        full_key.extend_from_slice(&key_bytes);

        Ok(full_key)
    }

    /// Get list for a key
    pub async fn get(&self, key: &K) -> Result<Vec<V>> {
        let encoded_key = self.encode_key(key)?;
        match self.backend.get(&encoded_key).await? {
            Some(bytes) => {
                let list = serde_json::from_slice(&bytes).map_err(|e| {
                    StreamingError::StateBackend(format!("Failed to decode list: {}", e))
                })?;
                Ok(list)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Add an item to the list
    pub async fn add(&self, key: &K, value: V) -> Result<()> {
        let mut list = self.get(key).await?;
        list.push(value);

        let encoded_key = self.encode_key(key)?;
        let encoded_value = serde_json::to_vec(&list).map_err(|e| {
            StreamingError::StateBackend(format!("Failed to encode list: {}", e))
        })?;

        self.backend.put(encoded_key, encoded_value).await
    }

    /// Clear the list for a key
    pub async fn clear(&self, key: &K) -> Result<()> {
        let encoded_key = self.encode_key(key)?;
        self.backend.delete(&encoded_key).await
    }
}

/// Map state that stores a map of values per key
pub struct MapState<K, MK, MV> {
    backend: Arc<dyn StateBackend>,
    namespace: String,
    _phantom: std::marker::PhantomData<(K, MK, MV)>,
}

impl<K, MK, MV> MapState<K, MK, MV>
where
    K: Serialize + for<'de> Deserialize<'de>,
    MK: Serialize + for<'de> Deserialize<'de> + Hash + Eq,
    MV: Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new map state
    pub fn new(backend: Arc<dyn StateBackend>, namespace: impl Into<String>) -> Self {
        Self {
            backend,
            namespace: namespace.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    fn encode_key(&self, key: &K) -> Result<Vec<u8>> {
        let key_bytes = serde_json::to_vec(key).map_err(|e| {
            StreamingError::StateBackend(format!("Failed to encode key: {}", e))
        })?;

        let mut full_key = self.namespace.as_bytes().to_vec();
        full_key.push(b':');
        full_key.extend_from_slice(&key_bytes);

        Ok(full_key)
    }

    /// Get the entire map for a key
    pub async fn get(&self, key: &K) -> Result<HashMap<MK, MV>> {
        let encoded_key = self.encode_key(key)?;
        match self.backend.get(&encoded_key).await? {
            Some(bytes) => {
                let map = serde_json::from_slice(&bytes).map_err(|e| {
                    StreamingError::StateBackend(format!("Failed to decode map: {}", e))
                })?;
                Ok(map)
            }
            None => Ok(HashMap::new()),
        }
    }

    /// Get a specific entry from the map
    pub async fn get_entry(&self, key: &K, map_key: &MK) -> Result<Option<MV>> {
        let map = self.get(key).await?;
        Ok(map.get(map_key).cloned())
    }

    /// Put an entry into the map
    pub async fn put_entry(&self, key: &K, map_key: MK, map_value: MV) -> Result<()> {
        let mut map = self.get(key).await?;
        map.insert(map_key, map_value);

        let encoded_key = self.encode_key(key)?;
        let encoded_value = serde_json::to_vec(&map).map_err(|e| {
            StreamingError::StateBackend(format!("Failed to encode map: {}", e))
        })?;

        self.backend.put(encoded_key, encoded_value).await
    }

    /// Remove an entry from the map
    pub async fn remove_entry(&self, key: &K, map_key: &MK) -> Result<()> {
        let mut map = self.get(key).await?;
        map.remove(map_key);

        let encoded_key = self.encode_key(key)?;
        let encoded_value = serde_json::to_vec(&map).map_err(|e| {
            StreamingError::StateBackend(format!("Failed to encode map: {}", e))
        })?;

        self.backend.put(encoded_key, encoded_value).await
    }

    /// Clear the map for a key
    pub async fn clear(&self, key: &K) -> Result<()> {
        let encoded_key = self.encode_key(key)?;
        self.backend.delete(&encoded_key).await
    }
}

/// Reducing state that aggregates values
pub struct ReducingState<K, V> {
    value_state: ValueState<K, V>,
    reduce_fn: Arc<dyn Fn(&V, &V) -> V + Send + Sync>,
}

impl<K, V> ReducingState<K, V>
where
    K: Serialize + for<'de> Deserialize<'de>,
    V: Serialize + for<'de> Deserialize<'de> + Clone,
{
    /// Create a new reducing state
    pub fn new<F>(backend: Arc<dyn StateBackend>, namespace: impl Into<String>, reduce_fn: F) -> Self
    where
        F: Fn(&V, &V) -> V + Send + Sync + 'static,
    {
        Self {
            value_state: ValueState::new(backend, namespace),
            reduce_fn: Arc::new(reduce_fn),
        }
    }

    /// Add a value to the reducing state
    pub async fn add(&self, key: &K, value: V) -> Result<()> {
        let current = self.value_state.get(key).await?;
        let new_value = match current {
            Some(current) => (self.reduce_fn)(&current, &value),
            None => value,
        };
        self.value_state.put(key, &new_value).await
    }

    /// Get the current reduced value
    pub async fn get(&self, key: &K) -> Result<Option<V>> {
        self.value_state.get(key).await
    }

    /// Clear the state
    pub async fn clear(&self, key: &K) -> Result<()> {
        self.value_state.delete(key).await
    }
}

/// State descriptor for creating state instances
pub struct StateDescriptor<T> {
    pub name: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> StateDescriptor<T> {
    /// Create a new state descriptor
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Shared state context for operators
pub struct StateContext {
    backend: Arc<dyn StateBackend>,
}

impl StateContext {
    /// Create a new state context
    pub fn new(backend: Arc<dyn StateBackend>) -> Self {
        Self { backend }
    }

    /// Create a value state
    pub fn value_state<K, V>(&self, descriptor: &StateDescriptor<V>) -> ValueState<K, V>
    where
        K: Serialize + for<'de> Deserialize<'de>,
        V: Serialize + for<'de> Deserialize<'de>,
    {
        ValueState::new(self.backend.clone(), &descriptor.name)
    }

    /// Create a list state
    pub fn list_state<K, V>(&self, descriptor: &StateDescriptor<Vec<V>>) -> ListState<K, V>
    where
        K: Serialize + for<'de> Deserialize<'de>,
        V: Serialize + for<'de> Deserialize<'de>,
    {
        ListState::new(self.backend.clone(), &descriptor.name)
    }

    /// Create a map state
    pub fn map_state<K, MK, MV>(
        &self,
        descriptor: &StateDescriptor<HashMap<MK, MV>>,
    ) -> MapState<K, MK, MV>
    where
        K: Serialize + for<'de> Deserialize<'de>,
        MK: Serialize + for<'de> Deserialize<'de> + Hash + Eq,
        MV: Serialize + for<'de> Deserialize<'de>,
    {
        MapState::new(self.backend.clone(), &descriptor.name)
    }

    /// Create a reducing state
    pub fn reducing_state<K, V, F>(
        &self,
        descriptor: &StateDescriptor<V>,
        reduce_fn: F,
    ) -> ReducingState<K, V>
    where
        K: Serialize + for<'de> Deserialize<'de>,
        V: Serialize + for<'de> Deserialize<'de> + Clone,
        F: Fn(&V, &V) -> V + Send + Sync + 'static,
    {
        ReducingState::new(self.backend.clone(), &descriptor.name, reduce_fn)
    }

    /// Get the backend
    pub fn backend(&self) -> Arc<dyn StateBackend> {
        self.backend.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_value_state() {
        let backend = Arc::new(MemoryStateBackend::new());
        let state: ValueState<String, i32> = ValueState::new(backend, "test");

        assert_eq!(state.get(&"key1".to_string()).await.unwrap(), None);

        state.put(&"key1".to_string(), &42).await.unwrap();
        assert_eq!(state.get(&"key1".to_string()).await.unwrap(), Some(42));

        state.delete(&"key1".to_string()).await.unwrap();
        assert_eq!(state.get(&"key1".to_string()).await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_list_state() {
        let backend = Arc::new(MemoryStateBackend::new());
        let state: ListState<String, i32> = ListState::new(backend, "test");

        assert_eq!(state.get(&"key1".to_string()).await.unwrap(), Vec::<i32>::new());

        state.add(&"key1".to_string(), 1).await.unwrap();
        state.add(&"key1".to_string(), 2).await.unwrap();
        state.add(&"key1".to_string(), 3).await.unwrap();

        assert_eq!(state.get(&"key1".to_string()).await.unwrap(), vec![1, 2, 3]);

        state.clear(&"key1".to_string()).await.unwrap();
        assert_eq!(state.get(&"key1".to_string()).await.unwrap(), Vec::<i32>::new());
    }

    #[tokio::test]
    async fn test_reducing_state() {
        let backend = Arc::new(MemoryStateBackend::new());
        let state = ReducingState::new(backend, "test", |a: &i32, b: &i32| a + b);

        state.add(&"key1".to_string(), 10).await.unwrap();
        state.add(&"key1".to_string(), 20).await.unwrap();
        state.add(&"key1".to_string(), 30).await.unwrap();

        assert_eq!(state.get(&"key1".to_string()).await.unwrap(), Some(60));
    }
}
