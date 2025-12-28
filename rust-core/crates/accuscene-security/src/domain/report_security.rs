//! Report access control

use crate::auth::AuthContext;
use crate::error::{Result, SecurityError};

/// Check if user can access report
pub fn can_access_report(context: &AuthContext, report_id: &str, is_published: bool) -> Result<()> {
    // Admin can access all reports
    if context.is_admin() {
        return Ok(());
    }

    // Anyone can access published reports
    if is_published && context.has_permission("reports:read") {
        return Ok(());
    }

    // Check for report access permission
    if context.has_permission("reports:read") {
        return Ok(());
    }

    Err(SecurityError::ReportAccessDenied {
        report_id: report_id.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_published_report_access() {
        let context = AuthContext {
            user_id: "user".to_string(),
            session_id: None,
            roles: vec![],
            permissions: vec!["reports:read".to_string()],
            mfa_verified: true,
            session_metadata: None,
        };

        assert!(can_access_report(&context, "report-123", true).is_ok());
    }
}
