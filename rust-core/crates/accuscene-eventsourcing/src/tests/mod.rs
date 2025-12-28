//! Comprehensive tests for the event sourcing system.

#![cfg(test)]

use crate::prelude::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Test aggregate for integration tests.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestCase {
    id: String,
    version: u64,
    title: String,
    status: String,
    events: Vec<TestCaseEvent>,
}

/// Test event for the test aggregate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum TestCaseEvent {
    Created { title: String },
    Updated { title: String },
    StatusChanged { status: String },
}

impl Event for TestCaseEvent {
    fn event_type(&self) -> &'static str {
        match self {
            TestCaseEvent::Created { .. } => "TestCaseCreated",
            TestCaseEvent::Updated { .. } => "TestCaseUpdated",
            TestCaseEvent::StatusChanged { .. } => "TestCaseStatusChanged",
        }
    }

    fn aggregate_id(&self) -> &str {
        "test-case"
    }

    fn aggregate_type(&self) -> &'static str {
        "TestCase"
    }
}

#[async_trait::async_trait]
impl Aggregate for TestCase {
    type Id = String;
    type Event = TestCaseEvent;

    fn aggregate_type() -> &'static str {
        "TestCase"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn apply(&mut self, event: &Self::Event) -> Result<()> {
        match event {
            TestCaseEvent::Created { title } => {
                self.title = title.clone();
            }
            TestCaseEvent::Updated { title } => {
                self.title = title.clone();
            }
            TestCaseEvent::StatusChanged { status } => {
                self.status = status.clone();
            }
        }
        self.events.push(event.clone());
        Ok(())
    }

    async fn handle(&self, _command: Box<dyn std::any::Any + Send>) -> Result<Vec<Self::Event>> {
        Ok(vec![])
    }

    fn default_state(id: Self::Id) -> Self {
        Self {
            id,
            version: 0,
            title: String::new(),
            status: "draft".to_string(),
            events: Vec::new(),
        }
    }
}

mod event_store_tests {
    use super::*;
    use crate::store::memory::InMemoryEventStore;

    #[tokio::test]
    async fn test_in_memory_event_store_workflow() {
        let store = InMemoryEventStore::new();

        // Create events
        let event1 = TestCaseEvent::Created {
            title: "Test Case 1".to_string(),
        };
        let event2 = TestCaseEvent::Updated {
            title: "Test Case 1 Updated".to_string(),
        };

        let envelope1 = EventEnvelope::new(event1.clone(), 1);
        let envelope2 = EventEnvelope::new(event2.clone(), 2);

        // Append events
        store.append_events(vec![envelope1, envelope2]).await.unwrap();

        // Load events
        let loaded: Vec<EventEnvelope<TestCaseEvent>> =
            store.load_all_events("test-case").await.unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].payload, event1);
        assert_eq!(loaded[1].payload, event2);

        // Check version
        let version = store.get_version("test-case").await.unwrap();
        assert_eq!(version, 2);
    }

    #[tokio::test]
    async fn test_event_store_sequence_validation() {
        let store = InMemoryEventStore::new();

        let event = TestCaseEvent::Created {
            title: "Test".to_string(),
        };

        // Try to append with wrong sequence
        let result = store
            .append_events(vec![EventEnvelope::new(event, 10)])
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_event_store_range_loading() {
        let store = InMemoryEventStore::new();

        // Create 5 events
        for i in 1..=5 {
            let event = TestCaseEvent::Updated {
                title: format!("Update {}", i),
            };
            store
                .append_events(vec![EventEnvelope::new(event, i)])
                .await
                .unwrap();
        }

        // Load range
        let events: Vec<EventEnvelope<TestCaseEvent>> =
            store.load_events_range("test-case", 2, 4).await.unwrap();

        assert_eq!(events.len(), 3);
    }
}

mod aggregate_tests {
    use super::*;

    #[test]
    fn test_aggregate_state_management() {
        let aggregate = TestCase::default_state("test-1".to_string());
        let mut state = AggregateState::new(aggregate);

        assert_eq!(state.version(), 0);
        assert!(!state.has_uncommitted_events());

        let event = TestCaseEvent::Created {
            title: "Test Case".to_string(),
        };

        state.apply_event(event).unwrap();

        assert_eq!(state.version(), 1);
        assert!(state.has_uncommitted_events());

        let uncommitted = state.take_uncommitted_events();
        assert_eq!(uncommitted.len(), 1);
        assert!(!state.has_uncommitted_events());
    }

    #[test]
    fn test_aggregate_event_application() {
        let aggregate = TestCase::default_state("test-1".to_string());
        let mut state = AggregateState::new(aggregate);

        let event1 = TestCaseEvent::Created {
            title: "Initial".to_string(),
        };
        let event2 = TestCaseEvent::Updated {
            title: "Updated".to_string(),
        };

        state.apply_events(vec![event1, event2]).unwrap();

        assert_eq!(state.aggregate.title, "Updated");
        assert_eq!(state.version(), 2);
    }
}

mod projection_tests {
    use super::*;
    use crate::projection::InMemoryProjection;

    #[test]
    fn test_in_memory_projection() {
        let projection = InMemoryProjection::<String, String>::new("test");

        projection.insert("key1".to_string(), "value1".to_string());
        projection.insert("key2".to_string(), "value2".to_string());

        assert_eq!(projection.len(), 2);
        assert_eq!(
            projection.get(&"key1".to_string()),
            Some("value1".to_string())
        );

        projection.remove(&"key1".to_string());
        assert_eq!(projection.len(), 1);

        projection.clear();
        assert!(projection.is_empty());
    }

    #[tokio::test]
    async fn test_projection_manager() {
        let manager = ProjectionManager::new();

        assert_eq!(manager.get_all_states().len(), 0);

        manager.stop("non-existent").ok();
        manager.start("non-existent").ok();
    }
}

mod snapshot_tests {
    use super::*;
    use crate::snapshot::{InMemorySnapshotStore, Snapshot};

    #[tokio::test]
    async fn test_snapshot_store() {
        let store = InMemorySnapshotStore::<TestCase>::new();

        let aggregate = TestCase {
            id: "test-1".to_string(),
            version: 5,
            title: "Test".to_string(),
            status: "open".to_string(),
            events: Vec::new(),
        };

        let snapshot = Snapshot::new(aggregate.clone(), 5);
        store.save(&snapshot).await.unwrap();

        let loaded = store.load("test-1").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().version, 5);

        let at_version = store.load_at_version("test-1", 3).await.unwrap();
        assert!(at_version.is_none());

        let list = store.list("test-1").await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_snapshot_strategy() {
        use crate::snapshot::SnapshotStrategy;

        let strategy = SnapshotStrategy::EveryNEvents(10);
        assert!(!strategy.should_snapshot(5, 0));
        assert!(strategy.should_snapshot(10, 0));
        assert!(strategy.should_snapshot(25, 10));

        let never = SnapshotStrategy::Never;
        assert!(!never.should_snapshot(100, 0));

        let always = SnapshotStrategy::Always;
        assert!(always.should_snapshot(1, 0));
    }
}

mod bus_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct TestSubscriber {
        count: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl crate::bus::EventSubscriber for TestSubscriber {
        async fn handle_event(&self, _event_data: &[u8], _event_type: &str) -> Result<()> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn interested_in(&self) -> Vec<&'static str> {
            vec!["TestCaseCreated", "TestCaseUpdated"]
        }
    }

    #[tokio::test]
    async fn test_event_bus_subscription() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let subscriber = Arc::new(TestSubscriber {
            count: Arc::clone(&counter),
        });

        bus.subscribe(subscriber);

        let event = TestCaseEvent::Created {
            title: "Test".to_string(),
        };

        let envelope = EventEnvelope::new(event, 1);
        bus.publish(&envelope).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        assert!(counter.load(Ordering::SeqCst) > 0);
        assert_eq!(bus.subscriber_count("TestCaseCreated"), 1);
    }

    #[test]
    fn test_command_bus() {
        let bus = CommandBus::new();
        assert_eq!(bus.handler_count(), 0);
        assert!(!bus.has_handler("TestCommand"));
    }
}

mod saga_tests {
    use super::*;
    use crate::saga::{SagaBuilder, SagaInstance, SagaManager, SagaStep};

    #[test]
    fn test_saga_instance_lifecycle() {
        let correlation_id = uuid::Uuid::new_v4();
        let mut instance = SagaInstance::new("TestSaga", correlation_id);

        assert_eq!(instance.state, SagaState::Starting);
        assert!(!instance.is_terminal());

        instance.record_step("step1");
        instance.record_command("command1");

        assert_eq!(instance.steps_completed.len(), 1);

        instance.complete();
        assert_eq!(instance.state, SagaState::Completed);
        assert!(instance.is_terminal());

        assert!(instance.duration().is_some());
    }

    #[test]
    fn test_saga_builder() {
        let step = SagaStep::new("step1", "command1")
            .with_compensation("undo1")
            .wait_for("Event1");

        let saga = SagaBuilder::new("TestSaga").add_step(step);

        assert_eq!(saga.saga_type(), "TestSaga");
        assert_eq!(saga.steps().len(), 1);
    }

    #[tokio::test]
    async fn test_saga_manager() {
        let manager = SagaManager::new();
        assert_eq!(manager.total_instances(), 0);

        let active = manager.list_active();
        assert_eq!(active.len(), 0);

        let completed = manager.list_completed();
        assert_eq!(completed.len(), 0);
    }
}

mod command_query_tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestCommand {
        id: String,
        title: String,
    }

    impl crate::command::Command for TestCommand {
        fn command_type(&self) -> &'static str {
            "TestCommand"
        }

        fn aggregate_id(&self) -> &str {
            &self.id
        }
    }

    #[test]
    fn test_command_envelope() {
        let command = TestCommand {
            id: "test-1".to_string(),
            title: "Test".to_string(),
        };

        let envelope = crate::command::CommandEnvelope::new(command)
            .with_issuer("user-123")
            .with_correlation_id(uuid::Uuid::new_v4());

        assert_eq!(envelope.command_type, "TestCommand");
        assert!(envelope.issuer.is_some());
        assert!(envelope.correlation_id.is_some());
    }

    #[derive(Debug, Clone)]
    struct TestQuery {
        id: String,
    }

    impl crate::query::Query for TestQuery {
        type Result = String;

        fn query_type(&self) -> &'static str {
            "TestQuery"
        }
    }

    #[test]
    fn test_query_envelope() {
        let query = TestQuery {
            id: "test-1".to_string(),
        };

        let envelope = crate::query::QueryEnvelope::new(query)
            .with_issuer("user-123");

        assert_eq!(envelope.query_type, "TestQuery");
        assert!(envelope.issuer.is_some());
    }

    #[test]
    fn test_pagination() {
        let pagination = crate::query::Pagination::new(2, 10).with_total(100);

        assert_eq!(pagination.offset(), 20);
        assert_eq!(pagination.limit(), 10);
        assert_eq!(pagination.total_pages(), Some(10));
        assert_eq!(pagination.has_next(), Some(true));
        assert!(pagination.has_previous());
    }
}

mod domain_events_tests {
    use super::*;
    use crate::domain::*;

    #[test]
    fn test_case_events() {
        let event = CaseCreated {
            case_id: "case-1".to_string(),
            case_number: "2024-001".to_string(),
            title: "Test Case".to_string(),
            description: None,
            created_by: "user-1".to_string(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(event.event_type(), "CaseCreated");
        assert_eq!(event.aggregate_type(), "Case");
    }

    #[test]
    fn test_scene_events() {
        let event = SceneCreated {
            scene_id: "scene-1".to_string(),
            case_id: "case-1".to_string(),
            name: "Test Scene".to_string(),
            description: None,
            location: None,
            created_by: "user-1".to_string(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(event.event_type(), "SceneCreated");
    }

    #[test]
    fn test_simulation_events() {
        let event = SimulationStarted {
            simulation_id: "sim-1".to_string(),
            scene_id: "scene-1".to_string(),
            simulation_type: SimulationType::Forward,
            physics_params: PhysicsParameters::default(),
            started_by: "user-1".to_string(),
            started_at: Utc::now(),
            config: HashMap::new(),
        };

        assert_eq!(event.event_type(), "SimulationStarted");
    }

    #[test]
    fn test_report_events() {
        let event = ReportGenerated {
            report_id: "report-1".to_string(),
            case_id: "case-1".to_string(),
            report_type: ReportType::FullReconstruction,
            title: "Test Report".to_string(),
            format: ReportFormat::Html,
            sections: vec![],
            generated_by: "user-1".to_string(),
            generated_at: Utc::now(),
            parameters: HashMap::new(),
        };

        assert_eq!(event.event_type(), "ReportGenerated");
    }
}

mod config_tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = crate::config::EventSourcingConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = crate::config::EventSourcingConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: crate::config::EventSourcingConfig =
            serde_json::from_str(&json).unwrap();
        assert!(deserialized.validate().is_ok());
    }
}

mod integration_tests {
    use super::*;
    use crate::store::memory::InMemoryEventStore;

    #[tokio::test]
    async fn test_full_event_sourcing_workflow() {
        // Create store
        let store = InMemoryEventStore::new();

        // Create aggregate
        let aggregate = TestCase::default_state("case-1".to_string());
        let mut state = AggregateState::new(aggregate);

        // Apply events
        let events = vec![
            TestCaseEvent::Created {
                title: "New Case".to_string(),
            },
            TestCaseEvent::Updated {
                title: "Updated Case".to_string(),
            },
            TestCaseEvent::StatusChanged {
                status: "open".to_string(),
            },
        ];

        state.apply_events(events).unwrap();

        // Save to store
        let uncommitted = state.take_uncommitted_events();
        let envelopes: Vec<_> = uncommitted
            .into_iter()
            .enumerate()
            .map(|(i, event)| EventEnvelope::new(event, (i + 1) as u64))
            .collect();

        store.append_events(envelopes).await.unwrap();

        // Load from store
        let loaded: Vec<EventEnvelope<TestCaseEvent>> =
            store.load_all_events("test-case").await.unwrap();

        assert_eq!(loaded.len(), 3);

        // Rebuild aggregate from events
        let mut rebuilt = TestCase::default_state("case-1".to_string());
        let mut rebuilt_state = AggregateState::new(rebuilt);
        rebuilt_state.load_from_history(loaded).unwrap();

        assert_eq!(rebuilt_state.aggregate.title, "Updated Case");
        assert_eq!(rebuilt_state.aggregate.status, "open");
        assert_eq!(rebuilt_state.version(), 3);
    }
}
