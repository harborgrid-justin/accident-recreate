//! Policy engine for authorization decisions
//!
//! Combines RBAC and ABAC into a unified authorization system.

use crate::error::{Result, SecurityError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::abac::{AbacService, Attributes, AuthorizationRequest};
use super::rbac::RbacService;

/// Policy engine combining RBAC and ABAC
pub struct PolicyEngine {
    rbac: RbacService,
    abac: AbacService,
    config: PolicyEngineConfig,
    decision_cache: HashMap<String, CachedDecision>,
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new(config: PolicyEngineConfig) -> Self {
        Self {
            rbac: RbacService::new(),
            abac: AbacService::new(),
            config,
            decision_cache: HashMap::new(),
        }
    }

    /// Get RBAC service
    pub fn rbac(&self) -> &RbacService {
        &self.rbac
    }

    /// Get mutable RBAC service
    pub fn rbac_mut(&mut self) -> &mut RbacService {
        &mut self.rbac
    }

    /// Get ABAC service
    pub fn abac(&self) -> &AbacService {
        &self.abac
    }

    /// Get mutable ABAC service
    pub fn abac_mut(&mut self) -> &mut AbacService {
        &mut self.abac
    }

    /// Make an authorization decision
    pub fn authorize(&mut self, request: PolicyRequest) -> Result<PolicyDecision> {
        // Check cache if enabled
        if self.config.cache_enabled {
            let cache_key = self.generate_cache_key(&request);
            if let Some(cached) = self.decision_cache.get(&cache_key) {
                if !cached.is_expired(self.config.cache_ttl_secs) {
                    return Ok(cached.decision.clone());
                }
            }
        }

        // Evaluate RBAC first (fast path)
        let rbac_decision = if self.config.rbac_enabled {
            self.evaluate_rbac(&request)?
        } else {
            PolicyDecision::not_applicable("RBAC disabled")
        };

        // If RBAC explicitly denies, short-circuit
        if matches!(rbac_decision.effect, DecisionEffect::Deny) {
            return Ok(rbac_decision);
        }

        // Evaluate ABAC for fine-grained control
        let abac_decision = if self.config.abac_enabled {
            self.evaluate_abac(&request)?
        } else {
            PolicyDecision::not_applicable("ABAC disabled")
        };

        // Combine decisions
        let final_decision = self.combine_decisions(&rbac_decision, &abac_decision);

        // Cache decision if enabled
        if self.config.cache_enabled {
            let cache_key = self.generate_cache_key(&request);
            self.decision_cache.insert(
                cache_key,
                CachedDecision {
                    decision: final_decision.clone(),
                    cached_at: chrono::Utc::now(),
                },
            );
        }

        Ok(final_decision)
    }

    /// Evaluate RBAC
    fn evaluate_rbac(&self, request: &PolicyRequest) -> Result<PolicyDecision> {
        // Check if user has required permission
        if self.rbac.has_permission(&request.user_id, &request.permission) {
            Ok(PolicyDecision {
                effect: DecisionEffect::Allow,
                reason: format!("RBAC: User has permission '{}'", request.permission),
                evaluated_by: vec!["RBAC".to_string()],
                metadata: HashMap::new(),
            })
        } else {
            Ok(PolicyDecision {
                effect: DecisionEffect::NotApplicable,
                reason: "RBAC: No matching permission".to_string(),
                evaluated_by: vec!["RBAC".to_string()],
                metadata: HashMap::new(),
            })
        }
    }

    /// Evaluate ABAC
    fn evaluate_abac(&self, request: &PolicyRequest) -> Result<PolicyDecision> {
        let authz_request = AuthorizationRequest {
            subject: request.subject_attrs.clone(),
            resource: request.resource_attrs.clone(),
            context: request.context_attrs.clone(),
            action: request.action.clone(),
        };

        let abac_decision = self.abac.evaluate(&authz_request)?;

        Ok(PolicyDecision {
            effect: if abac_decision.allowed {
                DecisionEffect::Allow
            } else {
                DecisionEffect::Deny
            },
            reason: abac_decision.reason.unwrap_or_else(|| "ABAC evaluation".to_string()),
            evaluated_by: vec!["ABAC".to_string()],
            metadata: HashMap::new(),
        })
    }

    /// Combine RBAC and ABAC decisions
    fn combine_decisions(
        &self,
        rbac: &PolicyDecision,
        abac: &PolicyDecision,
    ) -> PolicyDecision {
        // Deny overrides
        if matches!(rbac.effect, DecisionEffect::Deny) {
            return rbac.clone();
        }
        if matches!(abac.effect, DecisionEffect::Deny) {
            return abac.clone();
        }

        // Allow if either allows
        if matches!(rbac.effect, DecisionEffect::Allow) {
            return rbac.clone();
        }
        if matches!(abac.effect, DecisionEffect::Allow) {
            return abac.clone();
        }

        // Default deny
        PolicyDecision {
            effect: DecisionEffect::Deny,
            reason: "No policy allowed access".to_string(),
            evaluated_by: vec!["RBAC".to_string(), "ABAC".to_string()],
            metadata: HashMap::new(),
        }
    }

    /// Generate cache key for a request
    fn generate_cache_key(&self, request: &PolicyRequest) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(request.user_id.as_bytes());
        hasher.update(request.permission.as_bytes());
        hasher.update(request.action.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Clear decision cache
    pub fn clear_cache(&mut self) {
        self.decision_cache.clear();
    }

    /// Clean up expired cache entries
    pub fn cleanup_cache(&mut self) -> usize {
        let before_count = self.decision_cache.len();
        let ttl = self.config.cache_ttl_secs;
        self.decision_cache
            .retain(|_, cached| !cached.is_expired(ttl));
        before_count - self.decision_cache.len()
    }
}

/// Policy engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEngineConfig {
    /// Enable RBAC
    pub rbac_enabled: bool,
    /// Enable ABAC
    pub abac_enabled: bool,
    /// Enable decision caching
    pub cache_enabled: bool,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

impl Default for PolicyEngineConfig {
    fn default() -> Self {
        Self {
            rbac_enabled: true,
            abac_enabled: true,
            cache_enabled: true,
            cache_ttl_secs: 300, // 5 minutes
        }
    }
}

/// Policy authorization request
#[derive(Debug, Clone)]
pub struct PolicyRequest {
    /// User ID
    pub user_id: String,
    /// Required permission
    pub permission: String,
    /// Action being performed
    pub action: String,
    /// Subject attributes
    pub subject_attrs: Attributes,
    /// Resource attributes
    pub resource_attrs: Attributes,
    /// Context attributes
    pub context_attrs: Attributes,
}

impl PolicyRequest {
    /// Create a simple request with just user ID and permission
    pub fn simple(user_id: String, permission: String) -> Self {
        Self {
            user_id,
            permission: permission.clone(),
            action: permission,
            subject_attrs: HashMap::new(),
            resource_attrs: HashMap::new(),
            context_attrs: HashMap::new(),
        }
    }

    /// Add subject attribute
    pub fn with_subject_attr(mut self, key: String, value: super::abac::AttributeValue) -> Self {
        self.subject_attrs.insert(key, value);
        self
    }

    /// Add resource attribute
    pub fn with_resource_attr(mut self, key: String, value: super::abac::AttributeValue) -> Self {
        self.resource_attrs.insert(key, value);
        self
    }

    /// Add context attribute
    pub fn with_context_attr(mut self, key: String, value: super::abac::AttributeValue) -> Self {
        self.context_attrs.insert(key, value);
        self
    }
}

/// Policy decision
#[derive(Debug, Clone)]
pub struct PolicyDecision {
    /// Decision effect
    pub effect: DecisionEffect,
    /// Reason for the decision
    pub reason: String,
    /// Which policies evaluated this
    pub evaluated_by: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl PolicyDecision {
    /// Create a not applicable decision
    pub fn not_applicable(reason: &str) -> Self {
        Self {
            effect: DecisionEffect::NotApplicable,
            reason: reason.to_string(),
            evaluated_by: vec![],
            metadata: HashMap::new(),
        }
    }

    /// Check if access is allowed
    pub fn is_allowed(&self) -> bool {
        matches!(self.effect, DecisionEffect::Allow)
    }

    /// Check if access is denied
    pub fn is_denied(&self) -> bool {
        matches!(self.effect, DecisionEffect::Deny)
    }

    /// Convert to Result
    pub fn to_result(&self) -> Result<()> {
        if self.is_allowed() {
            Ok(())
        } else {
            Err(SecurityError::AccessDenied(self.reason.clone()))
        }
    }
}

/// Policy decision effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionEffect {
    Allow,
    Deny,
    NotApplicable,
}

/// Cached policy decision
#[derive(Debug, Clone)]
struct CachedDecision {
    decision: PolicyDecision,
    cached_at: chrono::DateTime<chrono::Utc>,
}

impl CachedDecision {
    fn is_expired(&self, ttl_secs: u64) -> bool {
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(self.cached_at);
        age.num_seconds() as u64 > ttl_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_engine_creation() {
        let engine = PolicyEngine::new(PolicyEngineConfig::default());
        assert!(engine.config.rbac_enabled);
        assert!(engine.config.abac_enabled);
    }

    #[test]
    fn test_rbac_authorization() {
        let mut engine = PolicyEngine::new(PolicyEngineConfig::default());

        // Assign role with permission
        engine.rbac_mut().assign_role("user123", "admin").unwrap();

        // Create request
        let request = PolicyRequest::simple(
            "user123".to_string(),
            "users:read".to_string(),
        );

        // Authorize
        let decision = engine.authorize(request).unwrap();
        assert!(decision.is_allowed());
    }

    #[test]
    fn test_decision_caching() {
        let mut engine = PolicyEngine::new(PolicyEngineConfig::default());
        engine.rbac_mut().assign_role("user123", "admin").unwrap();

        let request = PolicyRequest::simple(
            "user123".to_string(),
            "users:read".to_string(),
        );

        // First request - not cached
        let _ = engine.authorize(request.clone()).unwrap();
        assert_eq!(engine.decision_cache.len(), 1);

        // Second request - should use cache
        let _ = engine.authorize(request).unwrap();
        assert_eq!(engine.decision_cache.len(), 1);
    }

    #[test]
    fn test_cache_cleanup() {
        let mut engine = PolicyEngine::new(PolicyEngineConfig {
            cache_ttl_secs: 0, // Expire immediately
            ..Default::default()
        });

        engine.rbac_mut().assign_role("user123", "admin").unwrap();

        let request = PolicyRequest::simple(
            "user123".to_string(),
            "users:read".to_string(),
        );

        engine.authorize(request).unwrap();
        assert_eq!(engine.decision_cache.len(), 1);

        std::thread::sleep(std::time::Duration::from_millis(10));

        let removed = engine.cleanup_cache();
        assert_eq!(removed, 1);
        assert_eq!(engine.decision_cache.len(), 0);
    }
}
