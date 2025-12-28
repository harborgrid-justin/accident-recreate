//! Secret rotation

use crate::error::Result;

/// Secret rotation policy
pub struct RotationPolicy {
    pub interval_days: u32,
    pub auto_rotate: bool,
}

impl Default for RotationPolicy {
    fn default() -> Self {
        Self {
            interval_days: 90,
            auto_rotate: true,
        }
    }
}

/// Check if secret needs rotation based on age
pub fn needs_rotation(created_at: chrono::DateTime<chrono::Utc>, policy: &RotationPolicy) -> bool {
    let age = chrono::Utc::now()
        .signed_duration_since(created_at)
        .num_days() as u32;
    age >= policy.interval_days
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_rotation() {
        let policy = RotationPolicy::default();
        let old = chrono::Utc::now() - chrono::Duration::days(100);
        assert!(needs_rotation(old, &policy));

        let recent = chrono::Utc::now() - chrono::Duration::days(10);
        assert!(!needs_rotation(recent, &policy));
    }
}
