//! Policy engine with rules for fine-grained access control

use super::permission::{Action, Resource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Policy effect (allow or deny)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    /// Allow the action
    Allow,
    /// Deny the action (takes precedence over Allow)
    Deny,
}

/// Condition operator for policy rules
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionOperator {
    /// Equals
    Equals,
    /// Not equals
    NotEquals,
    /// Greater than
    GreaterThan,
    /// Greater than or equals
    GreaterThanOrEquals,
    /// Less than
    LessThan,
    /// Less than or equals
    LessThanOrEquals,
    /// Contains (for strings and arrays)
    Contains,
    /// Not contains
    NotContains,
    /// In (value in list)
    In,
    /// Not in
    NotIn,
    /// Matches regex
    Matches,
    /// IP address in CIDR range
    IpInRange,
    /// Time between
    TimeBetween,
}

/// A condition in a policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// The key to check (e.g., "user.department", "resource.owner", "time.hour")
    pub key: String,
    /// The operator to use
    pub operator: ConditionOperator,
    /// The value(s) to compare against
    pub values: Vec<String>,
}

impl Condition {
    /// Create a new condition
    pub fn new(
        key: impl Into<String>,
        operator: ConditionOperator,
        values: Vec<String>,
    ) -> Self {
        Self {
            key: key.into(),
            operator,
            values,
        }
    }

    /// Evaluate the condition against a context
    pub fn evaluate(&self, context: &HashMap<String, String>) -> bool {
        let value = match context.get(&self.key) {
            Some(v) => v,
            None => return false,
        };

        match self.operator {
            ConditionOperator::Equals => {
                self.values.first().map_or(false, |v| value == v)
            }
            ConditionOperator::NotEquals => {
                self.values.first().map_or(true, |v| value != v)
            }
            ConditionOperator::GreaterThan => {
                self.values.first().map_or(false, |v| value > v)
            }
            ConditionOperator::GreaterThanOrEquals => {
                self.values.first().map_or(false, |v| value >= v)
            }
            ConditionOperator::LessThan => {
                self.values.first().map_or(false, |v| value < v)
            }
            ConditionOperator::LessThanOrEquals => {
                self.values.first().map_or(false, |v| value <= v)
            }
            ConditionOperator::Contains => {
                self.values.iter().any(|v| value.contains(v))
            }
            ConditionOperator::NotContains => {
                !self.values.iter().any(|v| value.contains(v))
            }
            ConditionOperator::In => {
                self.values.iter().any(|v| v == value)
            }
            ConditionOperator::NotIn => {
                !self.values.iter().any(|v| v == value)
            }
            ConditionOperator::Matches => {
                // Simple pattern matching (not full regex for security)
                self.values.iter().any(|pattern| {
                    let pattern = pattern.replace('*', ".*");
                    if let Ok(re) = regex::Regex::new(&pattern) {
                        re.is_match(value)
                    } else {
                        false
                    }
                })
            }
            ConditionOperator::IpInRange => {
                // Simplified IP range check (would need proper CIDR parsing in production)
                self.values.iter().any(|range| {
                    value.starts_with(&range[..range.rfind('.').unwrap_or(0)])
                })
            }
            ConditionOperator::TimeBetween => {
                // Simplified time check (would need proper time parsing in production)
                if self.values.len() >= 2 {
                    value >= &self.values[0] && value <= &self.values[1]
                } else {
                    false
                }
            }
        }
    }
}

/// A policy rule that defines access control logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule ID
    pub id: String,
    /// Rule description
    pub description: String,
    /// Effect (Allow or Deny)
    pub effect: Effect,
    /// Actions this rule applies to
    pub actions: Vec<Action>,
    /// Resources this rule applies to
    pub resources: Vec<Resource>,
    /// Conditions that must be met
    pub conditions: Vec<Condition>,
    /// Priority (higher priority rules are evaluated first)
    pub priority: i32,
}

impl PolicyRule {
    /// Create a new policy rule
    pub fn new(id: impl Into<String>, effect: Effect) -> Self {
        Self {
            id: id.into(),
            description: String::new(),
            effect,
            actions: Vec::new(),
            resources: Vec::new(),
            conditions: Vec::new(),
            priority: 0,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add an action
    pub fn with_action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    /// Add actions
    pub fn with_actions(mut self, actions: Vec<Action>) -> Self {
        self.actions.extend(actions);
        self
    }

    /// Add a resource
    pub fn with_resource(mut self, resource: Resource) -> Self {
        self.resources.push(resource);
        self
    }

    /// Add resources
    pub fn with_resources(mut self, resources: Vec<Resource>) -> Self {
        self.resources.extend(resources);
        self
    }

    /// Add a condition
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Check if the rule applies to the given action and resource
    pub fn applies_to(&self, action: &Action, resource: &Resource) -> bool {
        let action_matches = self.actions.is_empty()
            || self.actions.iter().any(|a| a == action || matches!(a, Action::Manage));

        let resource_matches = self.resources.is_empty()
            || self.resources.iter().any(|r| r.matches(resource));

        action_matches && resource_matches
    }

    /// Evaluate all conditions
    pub fn evaluate_conditions(&self, context: &HashMap<String, String>) -> bool {
        if self.conditions.is_empty() {
            return true;
        }

        self.conditions.iter().all(|c| c.evaluate(context))
    }
}

/// A policy is a collection of rules
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Policy {
    /// Policy ID
    pub id: String,
    /// Policy name
    pub name: String,
    /// Policy description
    pub description: String,
    /// Rules in this policy
    pub rules: Vec<PolicyRule>,
}

impl Policy {
    /// Create a new policy
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            rules: Vec::new(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a rule
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
        // Sort by priority (descending)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Evaluate the policy for a given action, resource, and context
    pub fn evaluate(
        &self,
        action: &Action,
        resource: &Resource,
        context: &HashMap<String, String>,
    ) -> Effect {
        // Default to deny
        let mut effect = Effect::Deny;

        for rule in &self.rules {
            if rule.applies_to(action, resource) && rule.evaluate_conditions(context) {
                match rule.effect {
                    Effect::Deny => return Effect::Deny, // Deny takes precedence
                    Effect::Allow => effect = Effect::Allow,
                }
            }
        }

        effect
    }
}

/// Predefined policies
impl Policy {
    /// Create a policy that allows owners to manage their own resources
    pub fn owner_policy() -> Self {
        let mut policy = Self::new("owner_policy", "Owner Policy")
            .with_description("Allows resource owners to manage their own resources");

        let rule = PolicyRule::new("owner_manage", Effect::Allow)
            .with_description("Owner can manage their own resources")
            .with_action(Action::Manage)
            .with_resource(Resource::new("*"))
            .with_condition(Condition::new(
                "resource.owner",
                ConditionOperator::Equals,
                vec!["${user.id}".to_string()],
            ))
            .with_priority(100);

        policy.add_rule(rule);
        policy
    }

    /// Create a policy for business hours access
    pub fn business_hours_policy() -> Self {
        let mut policy = Self::new("business_hours", "Business Hours Policy")
            .with_description("Restricts access to business hours");

        let rule = PolicyRule::new("business_hours_allow", Effect::Allow)
            .with_description("Allow access during business hours")
            .with_action(Action::Read)
            .with_resource(Resource::new("*"))
            .with_condition(Condition::new(
                "time.hour",
                ConditionOperator::TimeBetween,
                vec!["09".to_string(), "17".to_string()],
            ))
            .with_priority(50);

        policy.add_rule(rule);
        policy
    }

    /// Create a policy for IP-based access control
    pub fn ip_whitelist_policy(allowed_ranges: Vec<String>) -> Self {
        let mut policy = Self::new("ip_whitelist", "IP Whitelist Policy")
            .with_description("Restricts access to specific IP ranges");

        let rule = PolicyRule::new("ip_whitelist_allow", Effect::Allow)
            .with_description("Allow access from whitelisted IPs")
            .with_action(Action::Manage)
            .with_resource(Resource::new("*"))
            .with_condition(Condition::new(
                "request.ip",
                ConditionOperator::IpInRange,
                allowed_ranges,
            ))
            .with_priority(90);

        policy.add_rule(rule);
        policy
    }
}

// Temporary regex support for condition matching
mod regex {
    pub struct Regex {
        pattern: String,
    }

    impl Regex {
        pub fn new(pattern: &str) -> Result<Self, ()> {
            Ok(Self {
                pattern: pattern.to_string(),
            })
        }

        pub fn is_match(&self, text: &str) -> bool {
            // Simplified pattern matching
            if self.pattern.contains(".*") {
                let parts: Vec<&str> = self.pattern.split(".*").collect();
                parts.iter().all(|part| text.contains(part))
            } else {
                text.contains(&self.pattern)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_evaluation() {
        let mut context = HashMap::new();
        context.insert("user.role".to_string(), "admin".to_string());

        let condition = Condition::new(
            "user.role",
            ConditionOperator::Equals,
            vec!["admin".to_string()],
        );

        assert!(condition.evaluate(&context));
    }

    #[test]
    fn test_policy_rule_evaluation() {
        let rule = PolicyRule::new("test_rule", Effect::Allow)
            .with_action(Action::Read)
            .with_resource(Resource::new("scene"));

        assert!(rule.applies_to(&Action::Read, &Resource::new("scene")));
        assert!(!rule.applies_to(&Action::Write, &Resource::new("scene")));
    }

    #[test]
    fn test_policy_evaluation() {
        let mut policy = Policy::new("test_policy", "Test Policy");

        let rule = PolicyRule::new("allow_read", Effect::Allow)
            .with_action(Action::Read)
            .with_resource(Resource::new("scene"));

        policy.add_rule(rule);

        let context = HashMap::new();
        let effect = policy.evaluate(&Action::Read, &Resource::new("scene"), &context);
        assert_eq!(effect, Effect::Allow);

        let effect = policy.evaluate(&Action::Write, &Resource::new("scene"), &context);
        assert_eq!(effect, Effect::Deny);
    }

    #[test]
    fn test_deny_precedence() {
        let mut policy = Policy::new("test_policy", "Test Policy");

        let allow_rule = PolicyRule::new("allow_rule", Effect::Allow)
            .with_action(Action::Read)
            .with_resource(Resource::new("scene"))
            .with_priority(10);

        let deny_rule = PolicyRule::new("deny_rule", Effect::Deny)
            .with_action(Action::Read)
            .with_resource(Resource::new("scene"))
            .with_priority(20);

        policy.add_rule(allow_rule);
        policy.add_rule(deny_rule);

        let context = HashMap::new();
        let effect = policy.evaluate(&Action::Read, &Resource::new("scene"), &context);
        assert_eq!(effect, Effect::Deny);
    }
}
