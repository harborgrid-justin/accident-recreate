//! Input sanitization

use crate::error::{Result, SecurityError};

/// Sanitize HTML input
pub fn sanitize_html(input: &str) -> String {
    input
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('&', "&amp;")
}

/// Sanitize SQL input (basic)
pub fn sanitize_sql(input: &str) -> Result<String> {
    if input.contains("--") || input.contains(';') || input.to_lowercase().contains("drop") {
        return Err(SecurityError::ValidationFailed(
            "Potential SQL injection detected".to_string(),
        ));
    }
    Ok(input.to_string())
}

/// Sanitize filename
pub fn sanitize_filename(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_html() {
        let input = "<script>alert('xss')</script>";
        let sanitized = sanitize_html(input);
        assert!(!sanitized.contains('<'));
        assert!(!sanitized.contains('>'));
    }

    #[test]
    fn test_sanitize_sql() {
        assert!(sanitize_sql("SELECT * FROM users").is_ok());
        assert!(sanitize_sql("DROP TABLE users").is_err());
    }

    #[test]
    fn test_sanitize_filename() {
        let sanitized = sanitize_filename("../../etc/passwd");
        assert!(!sanitized.contains('/'));
    }
}
