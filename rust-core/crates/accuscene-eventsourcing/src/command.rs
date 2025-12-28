//! Command handling for CQRS pattern.

use crate::error::{EventSourcingError, Result};
use crate::event::Event;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

/// Trait for commands in the CQRS pattern.
pub trait Command: Send + Sync + Debug + Clone {
    /// Returns the command type identifier.
    fn command_type(&self) -> &'static str;

    /// Returns the aggregate ID this command targets.
    fn aggregate_id(&self) -> &str;

    /// Validates the command.
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Command envelope with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEnvelope<C>
where
    C: Command,
{
    /// Unique command identifier.
    pub command_id: Uuid,

    /// Command type.
    pub command_type: String,

    /// Aggregate identifier.
    pub aggregate_id: String,

    /// The actual command payload.
    pub payload: C,

    /// User or service that issued the command.
    pub issuer: Option<String>,

    /// Correlation ID for tracking.
    pub correlation_id: Option<Uuid>,

    /// Timestamp when the command was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl<C> CommandEnvelope<C>
where
    C: Command,
{
    /// Creates a new command envelope.
    pub fn new(payload: C) -> Self {
        Self {
            command_id: Uuid::new_v4(),
            command_type: payload.command_type().to_string(),
            aggregate_id: payload.aggregate_id().to_string(),
            payload,
            issuer: None,
            correlation_id: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Sets the issuer.
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    /// Sets the correlation ID.
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Validates the command.
    pub fn validate(&self) -> Result<()> {
        self.payload.validate()
    }
}

/// Trait for command handlers.
#[async_trait]
pub trait CommandHandler<C>: Send + Sync
where
    C: Command,
{
    /// The event type produced by this handler.
    type Event: Event;

    /// Handles a command and returns the events to be persisted.
    async fn handle(&self, command: C) -> Result<Vec<Self::Event>>;

    /// Validates a command before handling.
    async fn validate(&self, command: &C) -> Result<()> {
        command.validate()
    }
}

/// Result of command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Command identifier.
    pub command_id: Uuid,

    /// Whether the command succeeded.
    pub success: bool,

    /// Number of events generated.
    pub events_generated: usize,

    /// Error message if failed.
    pub error: Option<String>,

    /// Aggregate version after command execution.
    pub version: Option<u64>,
}

impl CommandResult {
    /// Creates a successful result.
    pub fn success(command_id: Uuid, events_generated: usize, version: u64) -> Self {
        Self {
            command_id,
            success: true,
            events_generated,
            error: None,
            version: Some(version),
        }
    }

    /// Creates a failed result.
    pub fn failure(command_id: Uuid, error: impl Into<String>) -> Self {
        Self {
            command_id,
            success: false,
            events_generated: 0,
            error: Some(error.into()),
            version: None,
        }
    }
}

/// Command validator trait for complex validation logic.
#[async_trait]
pub trait CommandValidator<C>: Send + Sync
where
    C: Command,
{
    /// Validates a command.
    async fn validate(&self, command: &C) -> Result<()>;
}

/// Chain multiple validators together.
pub struct ValidatorChain<C>
where
    C: Command,
{
    validators: Vec<Box<dyn CommandValidator<C>>>,
}

impl<C> ValidatorChain<C>
where
    C: Command,
{
    /// Creates a new validator chain.
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Adds a validator to the chain.
    pub fn add_validator(mut self, validator: Box<dyn CommandValidator<C>>) -> Self {
        self.validators.push(validator);
        self
    }

    /// Validates a command through all validators.
    pub async fn validate(&self, command: &C) -> Result<()> {
        for validator in &self.validators {
            validator.validate(command).await?;
        }
        Ok(())
    }
}

impl<C> Default for ValidatorChain<C>
where
    C: Command,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Middleware for command processing.
#[async_trait]
pub trait CommandMiddleware: Send + Sync {
    /// Executes before command handling.
    async fn before(&self, command_id: &Uuid, command_type: &str) -> Result<()>;

    /// Executes after command handling.
    async fn after(&self, command_id: &Uuid, result: &CommandResult) -> Result<()>;

    /// Executes on command handling error.
    async fn on_error(&self, command_id: &Uuid, error: &EventSourcingError) -> Result<()>;
}

/// Command interceptor for cross-cutting concerns.
pub struct CommandInterceptor {
    middleware: Vec<Box<dyn CommandMiddleware>>,
}

impl CommandInterceptor {
    /// Creates a new command interceptor.
    pub fn new() -> Self {
        Self {
            middleware: Vec::new(),
        }
    }

    /// Adds middleware to the interceptor.
    pub fn add_middleware(mut self, middleware: Box<dyn CommandMiddleware>) -> Self {
        self.middleware.push(middleware);
        self
    }

    /// Executes before middleware.
    pub async fn before(&self, command_id: &Uuid, command_type: &str) -> Result<()> {
        for m in &self.middleware {
            m.before(command_id, command_type).await?;
        }
        Ok(())
    }

    /// Executes after middleware.
    pub async fn after(&self, command_id: &Uuid, result: &CommandResult) -> Result<()> {
        for m in &self.middleware {
            m.after(command_id, result).await?;
        }
        Ok(())
    }

    /// Executes error middleware.
    pub async fn on_error(&self, command_id: &Uuid, error: &EventSourcingError) -> Result<()> {
        for m in &self.middleware {
            m.on_error(command_id, error).await?;
        }
        Ok(())
    }
}

impl Default for CommandInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestCommand {
        id: String,
        value: i32,
    }

    impl Command for TestCommand {
        fn command_type(&self) -> &'static str {
            "TestCommand"
        }

        fn aggregate_id(&self) -> &str {
            &self.id
        }

        fn validate(&self) -> Result<()> {
            if self.value < 0 {
                return Err(EventSourcingError::CommandValidation(
                    "Value must be non-negative".to_string(),
                ));
            }
            Ok(())
        }
    }

    #[test]
    fn test_command_envelope() {
        let command = TestCommand {
            id: "test-1".to_string(),
            value: 42,
        };

        let envelope = CommandEnvelope::new(command.clone())
            .with_issuer("user-123")
            .with_correlation_id(Uuid::new_v4());

        assert_eq!(envelope.command_type, "TestCommand");
        assert_eq!(envelope.aggregate_id, "test-1");
        assert!(envelope.issuer.is_some());
        assert!(envelope.correlation_id.is_some());
    }

    #[test]
    fn test_command_validation() {
        let valid_command = TestCommand {
            id: "test-1".to_string(),
            value: 42,
        };

        assert!(valid_command.validate().is_ok());

        let invalid_command = TestCommand {
            id: "test-1".to_string(),
            value: -1,
        };

        assert!(invalid_command.validate().is_err());
    }

    #[test]
    fn test_command_result() {
        let success = CommandResult::success(Uuid::new_v4(), 3, 5);
        assert!(success.success);
        assert_eq!(success.events_generated, 3);
        assert_eq!(success.version, Some(5));

        let failure = CommandResult::failure(Uuid::new_v4(), "Test error");
        assert!(!failure.success);
        assert!(failure.error.is_some());
    }
}
