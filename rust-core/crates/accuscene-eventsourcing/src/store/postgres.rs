//! PostgreSQL event store implementation.

use crate::error::{EventSourcingError, Result};
use crate::event::{Event, EventEnvelope, SerializedEvent};
use crate::store::EventStore;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Row, Transaction};

/// PostgreSQL event store implementation.
#[derive(Debug, Clone)]
pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    /// Creates a new PostgreSQL event store.
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(EventSourcingError::database)?;

        Ok(Self { pool })
    }

    /// Creates a new PostgreSQL event store with a custom pool.
    pub fn with_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initializes the database schema.
    pub async fn initialize(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                event_id UUID PRIMARY KEY,
                event_type VARCHAR(255) NOT NULL,
                aggregate_id VARCHAR(255) NOT NULL,
                aggregate_type VARCHAR(255) NOT NULL,
                sequence BIGINT NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                correlation_id UUID,
                causation_id UUID,
                actor VARCHAR(255),
                metadata JSONB,
                payload_json JSONB NOT NULL,
                payload_binary BYTEA,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(aggregate_id, sequence)
            );

            CREATE INDEX IF NOT EXISTS idx_events_aggregate_id ON events(aggregate_id);
            CREATE INDEX IF NOT EXISTS idx_events_aggregate_type ON events(aggregate_type);
            CREATE INDEX IF NOT EXISTS idx_events_event_type ON events(event_type);
            CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_events_correlation_id ON events(correlation_id);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(EventSourcingError::database)?;

        Ok(())
    }

    /// Begins a transaction.
    pub async fn begin(&self) -> Result<Transaction<'_, Postgres>> {
        self.pool
            .begin()
            .await
            .map_err(EventSourcingError::database)
    }

    /// Returns the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl EventStore for PostgresEventStore {
    async fn append_events<E>(&self, events: Vec<EventEnvelope<E>>) -> Result<()>
    where
        E: Event + Serialize + Send + Sync,
    {
        if events.is_empty() {
            return Ok(());
        }

        let mut tx = self.begin().await?;

        for envelope in events {
            let serialized = SerializedEvent::from_envelope(&envelope)
                .map_err(EventSourcingError::serialization)?;

            let metadata_json = serde_json::to_value(&serialized.metadata.custom)
                .map_err(EventSourcingError::serialization)?;

            let payload_json: serde_json::Value = serde_json::from_str(&serialized.payload_json)
                .map_err(EventSourcingError::serialization)?;

            sqlx::query(
                r#"
                INSERT INTO events (
                    event_id, event_type, aggregate_id, aggregate_type,
                    sequence, timestamp, correlation_id, causation_id,
                    actor, metadata, payload_json, payload_binary
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                "#,
            )
            .bind(serialized.metadata.event_id)
            .bind(&serialized.metadata.event_type)
            .bind(&serialized.metadata.aggregate_id)
            .bind(&serialized.metadata.aggregate_type)
            .bind(serialized.metadata.sequence as i64)
            .bind(serialized.metadata.timestamp)
            .bind(serialized.metadata.correlation_id)
            .bind(serialized.metadata.causation_id)
            .bind(&serialized.metadata.actor)
            .bind(metadata_json)
            .bind(payload_json)
            .bind(serialized.payload_binary)
            .execute(&mut *tx)
            .await
            .map_err(EventSourcingError::database)?;
        }

        tx.commit().await.map_err(EventSourcingError::database)?;

        Ok(())
    }

    async fn load_events<E>(
        &self,
        aggregate_id: &str,
        from_sequence: u64,
    ) -> Result<Vec<EventEnvelope<E>>>
    where
        E: Event + for<'de> Deserialize<'de>,
    {
        let rows = sqlx::query(
            r#"
            SELECT event_id, event_type, aggregate_id, aggregate_type,
                   sequence, timestamp, correlation_id, causation_id,
                   actor, metadata, payload_json
            FROM events
            WHERE aggregate_id = $1 AND sequence >= $2
            ORDER BY sequence ASC
            "#,
        )
        .bind(aggregate_id)
        .bind(from_sequence as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(EventSourcingError::database)?;

        let mut result = Vec::new();

        for row in rows {
            let event_id: uuid::Uuid = row.get("event_id");
            let event_type: String = row.get("event_type");
            let aggregate_id: String = row.get("aggregate_id");
            let aggregate_type: String = row.get("aggregate_type");
            let sequence: i64 = row.get("sequence");
            let timestamp: chrono::DateTime<chrono::Utc> = row.get("timestamp");
            let correlation_id: Option<uuid::Uuid> = row.get("correlation_id");
            let causation_id: Option<uuid::Uuid> = row.get("causation_id");
            let actor: Option<String> = row.get("actor");
            let metadata: serde_json::Value = row.get("metadata");
            let payload_json: serde_json::Value = row.get("payload_json");

            let custom: std::collections::HashMap<String, String> =
                serde_json::from_value(metadata)
                    .map_err(EventSourcingError::deserialization)?;

            let metadata = crate::event::EventMetadata {
                event_id,
                event_type,
                aggregate_id,
                aggregate_type,
                sequence: sequence as u64,
                timestamp,
                correlation_id,
                causation_id,
                actor,
                custom,
            };

            let payload: E = serde_json::from_value(payload_json)
                .map_err(EventSourcingError::deserialization)?;

            result.push(EventEnvelope::with_metadata(payload, metadata));
        }

        Ok(result)
    }

    async fn get_version(&self, aggregate_id: &str) -> Result<u64> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(MAX(sequence), 0) as version
            FROM events
            WHERE aggregate_id = $1
            "#,
        )
        .bind(aggregate_id)
        .fetch_one(&self.pool)
        .await
        .map_err(EventSourcingError::database)?;

        let version: i64 = row.get("version");
        Ok(version as u64)
    }

    async fn load_events_range<E>(
        &self,
        aggregate_id: &str,
        from_sequence: u64,
        to_sequence: u64,
    ) -> Result<Vec<EventEnvelope<E>>>
    where
        E: Event + for<'de> Deserialize<'de>,
    {
        let rows = sqlx::query(
            r#"
            SELECT event_id, event_type, aggregate_id, aggregate_type,
                   sequence, timestamp, correlation_id, causation_id,
                   actor, metadata, payload_json
            FROM events
            WHERE aggregate_id = $1 AND sequence >= $2 AND sequence <= $3
            ORDER BY sequence ASC
            "#,
        )
        .bind(aggregate_id)
        .bind(from_sequence as i64)
        .bind(to_sequence as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(EventSourcingError::database)?;

        let mut result = Vec::new();

        for row in rows {
            let event_id: uuid::Uuid = row.get("event_id");
            let event_type: String = row.get("event_type");
            let aggregate_id: String = row.get("aggregate_id");
            let aggregate_type: String = row.get("aggregate_type");
            let sequence: i64 = row.get("sequence");
            let timestamp: chrono::DateTime<chrono::Utc> = row.get("timestamp");
            let correlation_id: Option<uuid::Uuid> = row.get("correlation_id");
            let causation_id: Option<uuid::Uuid> = row.get("causation_id");
            let actor: Option<String> = row.get("actor");
            let metadata: serde_json::Value = row.get("metadata");
            let payload_json: serde_json::Value = row.get("payload_json");

            let custom: std::collections::HashMap<String, String> =
                serde_json::from_value(metadata)
                    .map_err(EventSourcingError::deserialization)?;

            let metadata = crate::event::EventMetadata {
                event_id,
                event_type,
                aggregate_id,
                aggregate_type,
                sequence: sequence as u64,
                timestamp,
                correlation_id,
                causation_id,
                actor,
                custom,
            };

            let payload: E = serde_json::from_value(payload_json)
                .map_err(EventSourcingError::deserialization)?;

            result.push(EventEnvelope::with_metadata(payload, metadata));
        }

        Ok(result)
    }

    async fn stream_events_by_type<E>(&self, event_type: &str) -> Result<Vec<EventEnvelope<E>>>
    where
        E: Event + for<'de> Deserialize<'de>,
    {
        let rows = sqlx::query(
            r#"
            SELECT event_id, event_type, aggregate_id, aggregate_type,
                   sequence, timestamp, correlation_id, causation_id,
                   actor, metadata, payload_json
            FROM events
            WHERE event_type = $1
            ORDER BY timestamp ASC
            "#,
        )
        .bind(event_type)
        .fetch_all(&self.pool)
        .await
        .map_err(EventSourcingError::database)?;

        let mut result = Vec::new();

        for row in rows {
            let event_id: uuid::Uuid = row.get("event_id");
            let event_type: String = row.get("event_type");
            let aggregate_id: String = row.get("aggregate_id");
            let aggregate_type: String = row.get("aggregate_type");
            let sequence: i64 = row.get("sequence");
            let timestamp: chrono::DateTime<chrono::Utc> = row.get("timestamp");
            let correlation_id: Option<uuid::Uuid> = row.get("correlation_id");
            let causation_id: Option<uuid::Uuid> = row.get("causation_id");
            let actor: Option<String> = row.get("actor");
            let metadata: serde_json::Value = row.get("metadata");
            let payload_json: serde_json::Value = row.get("payload_json");

            let custom: std::collections::HashMap<String, String> =
                serde_json::from_value(metadata)
                    .map_err(EventSourcingError::deserialization)?;

            let metadata = crate::event::EventMetadata {
                event_id,
                event_type,
                aggregate_id,
                aggregate_type,
                sequence: sequence as u64,
                timestamp,
                correlation_id,
                causation_id,
                actor,
                custom,
            };

            let payload: E = serde_json::from_value(payload_json)
                .map_err(EventSourcingError::deserialization)?;

            result.push(EventEnvelope::with_metadata(payload, metadata));
        }

        Ok(result)
    }

    async fn delete_events(&self, aggregate_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM events WHERE aggregate_id = $1")
            .bind(aggregate_id)
            .execute(&self.pool)
            .await
            .map_err(EventSourcingError::database)?;

        Ok(())
    }
}
