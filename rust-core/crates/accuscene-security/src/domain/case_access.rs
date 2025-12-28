//! Case access control

use crate::auth::AuthContext;
use crate::error::{Result, SecurityError};

/// Check if user can access case
pub fn can_access_case(context: &AuthContext, case_id: &str, case_owner: &str) -> Result<()> {
    // Admin can access all cases
    if context.is_admin() {
        return Ok(());
    }

    // Owner can access their cases
    if context.user_id == case_owner {
        return Ok(());
    }

    // Check for case:read permission
    if context.has_permission("cases:read") {
        return Ok(());
    }

    Err(SecurityError::CaseAccessDenied {
        case_id: case_id.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::SessionMetadata;

    #[test]
    fn test_admin_access() {
        let context = AuthContext {
            user_id: "admin".to_string(),
            session_id: None,
            roles: vec!["admin".to_string()],
            permissions: vec![],
            mfa_verified: true,
            session_metadata: None,
        };

        assert!(can_access_case(&context, "case-123", "other-user").is_ok());
    }

    #[test]
    fn test_owner_access() {
        let context = AuthContext {
            user_id: "owner".to_string(),
            session_id: None,
            roles: vec![],
            permissions: vec![],
            mfa_verified: true,
            session_metadata: None,
        };

        assert!(can_access_case(&context, "case-123", "owner").is_ok());
    }
}
