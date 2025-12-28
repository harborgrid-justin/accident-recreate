//! Room-based subscription system for per-case collaboration.

use crate::error::{Result, StreamingError};
use crate::event::{Event, RoomId, UserId};
use crate::pubsub::{DefaultPubSub, PubSub, Subscriber};
use dashmap::{DashMap, DashSet};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Room information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    /// Room ID
    pub id: RoomId,
    /// Room name
    pub name: String,
    /// Room description
    pub description: Option<String>,
    /// Maximum number of users
    pub max_users: Option<usize>,
    /// Current user count
    pub user_count: usize,
    /// Room metadata
    pub metadata: Option<serde_json::Value>,
}

impl RoomInfo {
    /// Create a new room info
    pub fn new(id: impl Into<RoomId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            max_users: None,
            user_count: 0,
            metadata: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set max users
    pub fn with_max_users(mut self, max: usize) -> Self {
        self.max_users = Some(max);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Check if room is full
    pub fn is_full(&self) -> bool {
        if let Some(max) = self.max_users {
            self.user_count >= max
        } else {
            false
        }
    }
}

/// Room for managing subscriptions and users
pub struct Room {
    /// Room information
    info: RoomInfo,
    /// Users in the room
    users: Arc<DashSet<UserId>>,
    /// Pub/sub system for this room
    pubsub: Arc<dyn PubSub>,
}

impl Room {
    /// Create a new room
    pub fn new(info: RoomInfo) -> Self {
        Self {
            info,
            users: Arc::new(DashSet::new()),
            pubsub: Arc::new(DefaultPubSub::new()),
        }
    }

    /// Create a room with custom pub/sub
    pub fn with_pubsub(info: RoomInfo, pubsub: Arc<dyn PubSub>) -> Self {
        Self {
            info,
            users: Arc::new(DashSet::new()),
            pubsub,
        }
    }

    /// Get room ID
    pub fn id(&self) -> &RoomId {
        &self.info.id
    }

    /// Get room info
    pub fn info(&self) -> &RoomInfo {
        &self.info
    }

    /// Join the room
    pub async fn join(&self, user_id: impl Into<UserId>) -> Result<Box<dyn Subscriber>> {
        let user_id = user_id.into();

        // Check if room is full
        if self.info.is_full() {
            return Err(StreamingError::RoomNotFound(format!(
                "Room '{}' is full",
                self.info.id
            )));
        }

        // Add user
        self.users.insert(user_id.clone());

        // Subscribe to room events
        let subscriber = self.pubsub.subscribe(&self.info.id).await?;

        Ok(subscriber)
    }

    /// Leave the room
    pub async fn leave(&self, user_id: &UserId) -> Result<()> {
        self.users.remove(user_id);
        Ok(())
    }

    /// Publish an event to the room
    pub async fn publish(&self, event: Event) -> Result<()> {
        self.pubsub.publish(&self.info.id, event).await
    }

    /// Get users in the room
    pub fn get_users(&self) -> Vec<UserId> {
        self.users.iter().map(|u| u.clone()).collect()
    }

    /// Get user count
    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    /// Check if user is in room
    pub fn has_user(&self, user_id: &UserId) -> bool {
        self.users.contains(user_id)
    }
}

/// Room manager for managing multiple rooms
pub struct RoomManager {
    /// All rooms indexed by ID
    rooms: Arc<DashMap<RoomId, Arc<Room>>>,
    /// User to rooms mapping
    user_rooms: Arc<DashMap<UserId, DashSet<RoomId>>>,
}

impl RoomManager {
    /// Create a new room manager
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
            user_rooms: Arc::new(DashMap::new()),
        }
    }

    /// Create a new room
    pub async fn create_room(&self, info: RoomInfo) -> Result<Arc<Room>> {
        let room_id = info.id.clone();

        if self.rooms.contains_key(&room_id) {
            return Err(StreamingError::Internal(format!(
                "Room '{}' already exists",
                room_id
            )));
        }

        let room = Arc::new(Room::new(info));
        self.rooms.insert(room_id, room.clone());

        Ok(room)
    }

    /// Get a room
    pub fn get_room(&self, room_id: &RoomId) -> Result<Arc<Room>> {
        self.rooms
            .get(room_id)
            .map(|r| r.clone())
            .ok_or_else(|| StreamingError::RoomNotFound(room_id.clone()))
    }

    /// Delete a room
    pub async fn delete_room(&self, room_id: &RoomId) -> Result<()> {
        let room = self.get_room(room_id)?;

        // Remove all users from the room
        for user_id in room.get_users() {
            self.leave_room(room_id, &user_id).await?;
        }

        self.rooms.remove(room_id);

        Ok(())
    }

    /// Join a room
    pub async fn join_room(
        &self,
        room_id: &RoomId,
        user_id: impl Into<UserId>,
    ) -> Result<Box<dyn Subscriber>> {
        let user_id = user_id.into();
        let room = self.get_room(room_id)?;

        let subscriber = room.join(user_id.clone()).await?;

        // Track user's room membership
        self.user_rooms
            .entry(user_id.clone())
            .or_insert_with(DashSet::new)
            .insert(room_id.clone());

        Ok(subscriber)
    }

    /// Leave a room
    pub async fn leave_room(&self, room_id: &RoomId, user_id: &UserId) -> Result<()> {
        let room = self.get_room(room_id)?;
        room.leave(user_id).await?;

        // Remove from user's room membership
        if let Some(rooms) = self.user_rooms.get(user_id) {
            rooms.remove(room_id);
        }

        Ok(())
    }

    /// Leave all rooms for a user
    pub async fn leave_all_rooms(&self, user_id: &UserId) -> Result<()> {
        if let Some((_, rooms)) = self.user_rooms.remove(user_id) {
            for room_id in rooms.iter() {
                if let Ok(room) = self.get_room(&room_id) {
                    room.leave(user_id).await?;
                }
            }
        }

        Ok(())
    }

    /// Publish to a room
    pub async fn publish_to_room(&self, room_id: &RoomId, event: Event) -> Result<()> {
        let room = self.get_room(room_id)?;
        room.publish(event).await
    }

    /// Get rooms for a user
    pub fn get_user_rooms(&self, user_id: &UserId) -> Vec<RoomId> {
        self.user_rooms
            .get(user_id)
            .map(|rooms| rooms.iter().map(|r| r.clone()).collect())
            .unwrap_or_default()
    }

    /// List all rooms
    pub fn list_rooms(&self) -> Vec<RoomInfo> {
        self.rooms
            .iter()
            .map(|entry| {
                let room = entry.value();
                let mut info = room.info().clone();
                info.user_count = room.user_count();
                info
            })
            .collect()
    }

    /// Get total room count
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Clear all rooms
    pub async fn clear(&self) -> Result<()> {
        self.rooms.clear();
        self.user_rooms.clear();
        Ok(())
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_room_creation() {
        let info = RoomInfo::new("room1", "Test Room");
        let room = Room::new(info);

        assert_eq!(room.id(), "room1");
        assert_eq!(room.user_count(), 0);
    }

    #[tokio::test]
    async fn test_room_join_leave() {
        let info = RoomInfo::new("room1", "Test Room");
        let room = Room::new(info);

        let _subscriber = room.join("user1").await.unwrap();
        assert_eq!(room.user_count(), 1);
        assert!(room.has_user(&"user1".to_string()));

        room.leave(&"user1".to_string()).await.unwrap();
        assert_eq!(room.user_count(), 0);
    }

    #[tokio::test]
    async fn test_room_capacity() {
        let info = RoomInfo::new("room1", "Test Room").with_max_users(2);
        let room = Room::new(info);

        let _sub1 = room.join("user1").await.unwrap();
        let _sub2 = room.join("user2").await.unwrap();

        // Room should be full
        let result = room.join("user3").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_room_manager() {
        let manager = RoomManager::new();

        let info = RoomInfo::new("room1", "Test Room");
        manager.create_room(info).await.unwrap();

        assert_eq!(manager.room_count(), 1);

        let _subscriber = manager.join_room(&"room1".to_string(), "user1").await.unwrap();

        let user_rooms = manager.get_user_rooms(&"user1".to_string());
        assert_eq!(user_rooms.len(), 1);
        assert_eq!(user_rooms[0], "room1");
    }

    #[tokio::test]
    async fn test_leave_all_rooms() {
        let manager = RoomManager::new();

        manager
            .create_room(RoomInfo::new("room1", "Room 1"))
            .await
            .unwrap();
        manager
            .create_room(RoomInfo::new("room2", "Room 2"))
            .await
            .unwrap();

        manager.join_room(&"room1".to_string(), "user1").await.unwrap();
        manager.join_room(&"room2".to_string(), "user1").await.unwrap();

        assert_eq!(manager.get_user_rooms(&"user1".to_string()).len(), 2);

        manager.leave_all_rooms(&"user1".to_string()).await.unwrap();

        assert_eq!(manager.get_user_rooms(&"user1".to_string()).len(), 0);
    }
}
