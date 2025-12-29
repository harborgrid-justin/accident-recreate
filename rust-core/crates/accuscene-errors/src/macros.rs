//! Macros for convenient error creation and handling

/// Creates an AccuSceneError with automatic location tracking
///
/// # Examples
///
/// ```
/// use accuscene_errors::error;
///
/// let err = error!(Validation, "Invalid input");
/// let err = error!(Database, "Connection failed", details: "Timeout after 30s");
/// ```
#[macro_export]
macro_rules! error {
    // Simple: error!(Code, "message")
    ($code:ident, $msg:expr) => {
        $crate::AccuSceneError::$code($msg)
            .with_location(file!(), line!())
    };

    // With details: error!(Code, "message", details: "details")
    ($code:ident, $msg:expr, details: $details:expr) => {
        $crate::AccuSceneError::$code($msg)
            .with_details($details)
            .with_location(file!(), line!())
    };

    // With context: error!(Code, "message", context: "context")
    ($code:ident, $msg:expr, context: $ctx:expr) => {
        $crate::AccuSceneError::$code($msg)
            .with_context($ctx)
            .with_location(file!(), line!())
    };

    // With metadata: error!(Code, "message", metadata: { "key" => "value" })
    ($code:ident, $msg:expr, metadata: { $($key:expr => $value:expr),* }) => {
        {
            let mut err = $crate::AccuSceneError::$code($msg)
                .with_location(file!(), line!());
            $(
                err = err.with_metadata($key, $value);
            )*
            err
        }
    };
}

/// Ensures a condition is true, otherwise returns an error
///
/// # Examples
///
/// ```
/// use accuscene_errors::{ensure, Result};
///
/// fn validate_age(age: i32) -> Result<()> {
///     ensure!(age >= 0, Validation, "Age must be non-negative");
///     ensure!(age <= 150, Validation, "Age must be realistic");
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $code:ident, $msg:expr) => {
        if !$cond {
            return Err($crate::error!($code, $msg));
        }
    };

    ($cond:expr, $code:ident, $msg:expr, details: $details:expr) => {
        if !$cond {
            return Err($crate::error!($code, $msg, details: $details));
        }
    };
}

/// Logs an error and returns it
///
/// # Examples
///
/// ```
/// use accuscene_errors::{log_error, Result};
///
/// fn process() -> Result<()> {
///     Err(log_error!(Internal, "Processing failed"))
/// }
/// ```
#[macro_export]
macro_rules! log_error {
    ($code:ident, $msg:expr) => {{
        let err = $crate::error!($code, $msg);
        tracing::error!(
            error_id = %err.id(),
            error_code = %err.code(),
            message = %err.message(),
            "Error occurred"
        );
        err
    }};

    ($code:ident, $msg:expr, details: $details:expr) => {{
        let err = $crate::error!($code, $msg, details: $details);
        tracing::error!(
            error_id = %err.id(),
            error_code = %err.code(),
            message = %err.message(),
            details = %err.details().unwrap_or(""),
            "Error occurred"
        );
        err
    }};
}

/// Wraps a Result, adding context on error
///
/// # Examples
///
/// ```
/// use accuscene_errors::{wrap_err, Result};
///
/// fn load_config() -> Result<Config> {
///     wrap_err!(
///         std::fs::read_to_string("config.json"),
///         "Failed to load configuration"
///     )
/// }
/// ```
#[macro_export]
macro_rules! wrap_err {
    ($expr:expr, $ctx:expr) => {
        $expr.map_err(|e| {
            let err: $crate::AccuSceneError = e.into();
            err.with_context($ctx).with_location(file!(), line!())
        })
    };
}

/// Creates a bail macro that returns early with an error
///
/// # Examples
///
/// ```
/// use accuscene_errors::{bail, Result};
///
/// fn process(value: i32) -> Result<()> {
///     if value < 0 {
///         bail!(Validation, "Value must be positive");
///     }
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! bail {
    ($code:ident, $msg:expr) => {
        return Err($crate::error!($code, $msg))
    };

    ($code:ident, $msg:expr, details: $details:expr) => {
        return Err($crate::error!($code, $msg, details: $details))
    };
}

/// Tries an expression, converting errors and adding context
///
/// # Examples
///
/// ```
/// use accuscene_errors::{try_with_context, Result};
///
/// fn process() -> Result<()> {
///     let data = try_with_context!(
///         load_data(),
///         "Failed to load data during processing"
///     );
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! try_with_context {
    ($expr:expr, $ctx:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                let err: $crate::AccuSceneError = e.into();
                return Err(err.with_context($ctx).with_location(file!(), line!()));
            }
        }
    };
}

/// Creates a recoverable error
///
/// # Examples
///
/// ```
/// use accuscene_errors::recoverable_error;
///
/// let err = recoverable_error!(Network, "Connection timeout");
/// ```
#[macro_export]
macro_rules! recoverable_error {
    ($code:ident, $msg:expr) => {
        $crate::error!($code, $msg).with_recoverable(true)
    };
}

/// Creates a non-recoverable error
///
/// # Examples
///
/// ```
/// use accuscene_errors::fatal_error;
///
/// let err = fatal_error!(Internal, "Critical system failure");
/// ```
#[macro_export]
macro_rules! fatal_error {
    ($code:ident, $msg:expr) => {
        $crate::error!($code, $msg)
            .with_recoverable(false)
            .with_severity($crate::ErrorSeverity::Critical)
    };
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_error_macro() {
        let err = error!(validation, "Test error");
        assert_eq!(err.code(), ErrorCode::Validation);
        assert!(err.location().is_some());
    }

    #[test]
    fn test_error_macro_with_details() {
        let err = error!(database, "Connection failed", details: "Timeout");
        assert_eq!(err.details(), Some("Timeout"));
    }

    #[test]
    fn test_ensure_macro() {
        fn validate(value: i32) -> Result<()> {
            ensure!(value > 0, validation, "Value must be positive");
            Ok(())
        }

        assert!(validate(-1).is_err());
        assert!(validate(1).is_ok());
    }

    #[test]
    fn test_recoverable_error_macro() {
        let err = recoverable_error!(network, "Timeout");
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_fatal_error_macro() {
        let err = fatal_error!(internal, "Critical failure");
        assert!(!err.is_recoverable());
        assert_eq!(err.severity(), ErrorSeverity::Critical);
    }
}
