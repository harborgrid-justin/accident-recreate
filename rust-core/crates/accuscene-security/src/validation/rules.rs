//! Validation rules

use crate::error::{Result, SecurityError};
use regex::Regex;

/// Email validation
pub fn validate_email(email: &str) -> Result<()> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if email_regex.is_match(email) {
        Ok(())
    } else {
        Err(SecurityError::ValidationFailed("Invalid email".to_string()))
    }
}

/// URL validation
pub fn validate_url(url: &str) -> Result<()> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err(SecurityError::ValidationFailed("Invalid URL".to_string()))
    }
}

/// UUID validation
pub fn validate_uuid(uuid: &str) -> Result<()> {
    if uuid::Uuid::parse_str(uuid).is_ok() {
        Ok(())
    } else {
        Err(SecurityError::ValidationFailed("Invalid UUID".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid").is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("ftp://example.com").is_err());
    }
}
