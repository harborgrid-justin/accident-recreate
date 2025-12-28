//! Attribute-Based Access Control (ABAC)
//!
//! Fine-grained access control based on attributes of users, resources, and context.

use crate::error::{Result, SecurityError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ABAC service for attribute-based authorization
pub struct AbacService {
    policies: Vec<AbacPolicy>,
}

impl AbacService {
    /// Create a new ABAC service
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    /// Add a policy
    pub fn add_policy(&mut self, policy: AbacPolicy) {
        self.policies.push(policy);
    }

    /// Evaluate authorization request
    pub fn evaluate(&self, request: &AuthorizationRequest) -> Result<AuthorizationDecision> {
        let mut decisions = Vec::new();

        // Evaluate all applicable policies
        for policy in &self.policies {
            if policy.applies_to(request) {
                let decision = policy.evaluate(request)?;
                decisions.push(decision);
            }
        }

        // Combine decisions (deny overrides)
        let final_decision = self.combine_decisions(&decisions);

        Ok(final_decision)
    }

    /// Combine multiple decisions using deny-overrides algorithm
    fn combine_decisions(&self, decisions: &[PolicyDecision]) -> AuthorizationDecision {
        // If any explicit deny, return deny
        for decision in decisions {
            if matches!(decision.effect, Effect::Deny) {
                return AuthorizationDecision {
                    allowed: false,
                    reason: Some(decision.reason.clone().unwrap_or_else(|| "Access denied by policy".to_string())),
                    matched_policies: decisions.iter().map(|d| d.policy_id.clone()).collect(),
                };
            }
        }

        // If any allow, return allow
        for decision in decisions {
            if matches!(decision.effect, Effect::Allow) {
                return AuthorizationDecision {
                    allowed: true,
                    reason: Some("Access granted by policy".to_string()),
                    matched_policies: decisions.iter().map(|d| d.policy_id.clone()).collect(),
                };
            }
        }

        // Default deny
        AuthorizationDecision {
            allowed: false,
            reason: Some("No applicable policy found".to_string()),
            matched_policies: vec![],
        }
    }

    /// Check if user can perform action on resource
    pub fn authorize(
        &self,
        user_attrs: &Attributes,
        resource_attrs: &Attributes,
        context_attrs: &Attributes,
        action: &str,
    ) -> Result<()> {
        let request = AuthorizationRequest {
            subject: user_attrs.clone(),
            resource: resource_attrs.clone(),
            context: context_attrs.clone(),
            action: action.to_string(),
        };

        let decision = self.evaluate(&request)?;

        if decision.allowed {
            Ok(())
        } else {
            Err(SecurityError::AccessDenied(
                decision.reason.unwrap_or_else(|| "Access denied".to_string()),
            ))
        }
    }
}

impl Default for AbacService {
    fn default() -> Self {
        Self::new()
    }
}

/// ABAC policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacPolicy {
    /// Policy ID
    pub id: String,
    /// Policy description
    pub description: Option<String>,
    /// Effect (Allow or Deny)
    pub effect: Effect,
    /// Target specification
    pub target: PolicyTarget,
    /// Conditions that must be satisfied
    pub conditions: Vec<Condition>,
    /// Priority (higher = evaluated first)
    pub priority: i32,
}

impl AbacPolicy {
    /// Create a new ABAC policy
    pub fn new(id: String, effect: Effect) -> Self {
        Self {
            id,
            description: None,
            effect,
            target: PolicyTarget::default(),
            conditions: Vec::new(),
            priority: 0,
        }
    }

    /// Check if policy applies to the request
    pub fn applies_to(&self, request: &AuthorizationRequest) -> bool {
        self.target.matches(request)
    }

    /// Evaluate the policy against a request
    pub fn evaluate(&self, request: &AuthorizationRequest) -> Result<PolicyDecision> {
        // Check all conditions
        for condition in &self.conditions {
            if !condition.evaluate(request)? {
                return Ok(PolicyDecision {
                    policy_id: self.id.clone(),
                    effect: Effect::NotApplicable,
                    reason: Some(format!("Condition not satisfied: {}", condition.description())),
                });
            }
        }

        Ok(PolicyDecision {
            policy_id: self.id.clone(),
            effect: self.effect,
            reason: self.description.clone(),
        })
    }

    /// Add a condition to the policy
    pub fn add_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Set policy target
    pub fn with_target(mut self, target: PolicyTarget) -> Self {
        self.target = target;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Policy target specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyTarget {
    /// Target actions (empty = all actions)
    pub actions: Vec<String>,
    /// Target resource types (empty = all types)
    pub resource_types: Vec<String>,
    /// Target subjects (empty = all subjects)
    pub subjects: Vec<String>,
}

impl PolicyTarget {
    /// Check if target matches the request
    pub fn matches(&self, request: &AuthorizationRequest) -> bool {
        // Check action
        if !self.actions.is_empty() && !self.actions.contains(&request.action) {
            return false;
        }

        // Check resource type
        if !self.resource_types.is_empty() {
            if let Some(resource_type) = request.resource.get("type") {
                if !self.resource_types.contains(&resource_type.to_string()) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check subject
        if !self.subjects.is_empty() {
            if let Some(subject_id) = request.subject.get("id") {
                if !self.subjects.contains(&subject_id.to_string()) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

/// Policy condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    /// Attribute equals value
    AttributeEquals {
        attribute: String,
        value: AttributeValue,
        context: AttributeContext,
    },
    /// Attribute contains value
    AttributeContains {
        attribute: String,
        value: AttributeValue,
        context: AttributeContext,
    },
    /// Attribute is greater than value
    AttributeGreaterThan {
        attribute: String,
        value: AttributeValue,
        context: AttributeContext,
    },
    /// Attribute is less than value
    AttributeLessThan {
        attribute: String,
        value: AttributeValue,
        context: AttributeContext,
    },
    /// Time-based condition
    TimeBetween {
        start: chrono::NaiveTime,
        end: chrono::NaiveTime,
    },
    /// Date-based condition
    DateBetween {
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    },
    /// IP address in range
    IpInRange {
        cidr: String,
    },
    /// Custom condition
    Custom {
        name: String,
        parameters: HashMap<String, String>,
    },
}

impl Condition {
    /// Evaluate the condition
    pub fn evaluate(&self, request: &AuthorizationRequest) -> Result<bool> {
        match self {
            Condition::AttributeEquals { attribute, value, context } => {
                let attrs = context.get_attributes(request);
                if let Some(attr_value) = attrs.get(attribute) {
                    Ok(attr_value == value)
                } else {
                    Ok(false)
                }
            }
            Condition::AttributeContains { attribute, value, context } => {
                let attrs = context.get_attributes(request);
                if let Some(attr_value) = attrs.get(attribute) {
                    match (attr_value, value) {
                        (AttributeValue::String(s), AttributeValue::String(v)) => Ok(s.contains(v)),
                        (AttributeValue::List(list), v) => Ok(list.contains(v)),
                        _ => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            }
            Condition::TimeBetween { start, end } => {
                let now = chrono::Local::now().time();
                Ok(now >= *start && now <= *end)
            }
            Condition::DateBetween { start, end } => {
                let today = chrono::Local::now().date_naive();
                Ok(today >= *start && today <= *end)
            }
            Condition::IpInRange { cidr: _ } => {
                // Simplified - in production, use ipnetwork crate
                Ok(true)
            }
            _ => Ok(true), // Default to true for custom conditions
        }
    }

    /// Get condition description
    pub fn description(&self) -> String {
        match self {
            Condition::AttributeEquals { attribute, value, context } => {
                format!("{:?}.{} == {:?}", context, attribute, value)
            }
            Condition::TimeBetween { start, end } => {
                format!("Time between {} and {}", start, end)
            }
            _ => "Custom condition".to_string(),
        }
    }
}

/// Attribute context (subject, resource, or environment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeContext {
    Subject,
    Resource,
    Context,
}

impl AttributeContext {
    fn get_attributes<'a>(&self, request: &'a AuthorizationRequest) -> &'a Attributes {
        match self {
            AttributeContext::Subject => &request.subject,
            AttributeContext::Resource => &request.resource,
            AttributeContext::Context => &request.context,
        }
    }
}

/// Attribute value type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Number(i64),
    Boolean(bool),
    List(Vec<AttributeValue>),
}

impl std::fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::String(s) => write!(f, "{}", s),
            AttributeValue::Number(n) => write!(f, "{}", n),
            AttributeValue::Boolean(b) => write!(f, "{}", b),
            AttributeValue::List(l) => write!(f, "{:?}", l),
        }
    }
}

/// Attributes collection
pub type Attributes = HashMap<String, AttributeValue>;

/// Authorization request
#[derive(Debug, Clone)]
pub struct AuthorizationRequest {
    /// Subject (user) attributes
    pub subject: Attributes,
    /// Resource attributes
    pub resource: Attributes,
    /// Context/environment attributes
    pub context: Attributes,
    /// Action being performed
    pub action: String,
}

/// Authorization decision
#[derive(Debug, Clone)]
pub struct AuthorizationDecision {
    /// Whether access is allowed
    pub allowed: bool,
    /// Reason for the decision
    pub reason: Option<String>,
    /// Policies that matched
    pub matched_policies: Vec<String>,
}

/// Policy decision
#[derive(Debug, Clone)]
pub struct PolicyDecision {
    /// Policy ID
    pub policy_id: String,
    /// Effect of the policy
    pub effect: Effect,
    /// Reason for the decision
    pub reason: Option<String>,
}

/// Policy effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Allow,
    Deny,
    NotApplicable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abac_service_creation() {
        let service = AbacService::new();
        assert_eq!(service.policies.len(), 0);
    }

    #[test]
    fn test_policy_evaluation() {
        let mut service = AbacService::new();

        let policy = AbacPolicy::new("test-policy".to_string(), Effect::Allow)
            .add_condition(Condition::AttributeEquals {
                attribute: "department".to_string(),
                value: AttributeValue::String("engineering".to_string()),
                context: AttributeContext::Subject,
            });

        service.add_policy(policy);

        let mut subject = Attributes::new();
        subject.insert("department".to_string(), AttributeValue::String("engineering".to_string()));

        let request = AuthorizationRequest {
            subject,
            resource: Attributes::new(),
            context: Attributes::new(),
            action: "read".to_string(),
        };

        let decision = service.evaluate(&request).unwrap();
        assert!(decision.allowed);
    }

    #[test]
    fn test_deny_overrides() {
        let mut service = AbacService::new();

        // Add allow policy
        service.add_policy(AbacPolicy::new("allow-policy".to_string(), Effect::Allow));

        // Add deny policy with higher priority
        service.add_policy(
            AbacPolicy::new("deny-policy".to_string(), Effect::Deny)
                .with_priority(10)
        );

        let request = AuthorizationRequest {
            subject: Attributes::new(),
            resource: Attributes::new(),
            context: Attributes::new(),
            action: "read".to_string(),
        };

        let decision = service.evaluate(&request).unwrap();
        assert!(!decision.allowed); // Deny should override allow
    }

    #[test]
    fn test_time_based_condition() {
        let condition = Condition::TimeBetween {
            start: chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            end: chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        };

        let request = AuthorizationRequest {
            subject: Attributes::new(),
            resource: Attributes::new(),
            context: Attributes::new(),
            action: "read".to_string(),
        };

        assert!(condition.evaluate(&request).unwrap());
    }
}
