//! Error types and exit codes for cargo-doc-query

use std::process::ExitCode;
use thiserror::Error;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_cache_error_message() {
        let error = AppError::NoCache;
        assert!(error.to_string().contains("No cached index found"));
        assert_eq!(error.exit_code(), ExitCode::from(2));
    }

    #[test]
    fn test_not_found_error_message() {
        let error = AppError::NotFound("test::type".to_string());
        assert!(error.to_string().contains("test::type"));
        assert_eq!(error.exit_code(), ExitCode::from(3));
    }

    #[test]
    fn test_build_failed_error_message() {
        let error = AppError::BuildFailed("test failure".to_string());
        assert!(error.to_string().contains("test failure"));
        assert_eq!(error.exit_code(), ExitCode::from(4));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_error: AppError = io_error.into();
        assert!(matches!(app_error, AppError::Io(_)));
        assert_eq!(app_error.exit_code(), ExitCode::from(7));
    }

    #[test]
    fn test_json_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::InvalidData, "json parse error");
        let json_error = serde_json::Error::io(io_error);
        let app_error: AppError = json_error.into();
        assert!(matches!(app_error, AppError::Json(_)));
        assert_eq!(app_error.exit_code(), ExitCode::from(8));
    }

    #[test]
    fn test_config_error_message() {
        let error = AppError::Config("invalid configuration".to_string());
        assert!(error.to_string().contains("invalid configuration"));
        assert_eq!(error.exit_code(), ExitCode::from(9));
    }

    #[test]
    fn test_other_error_conversion() {
        let anyhow_error = anyhow::anyhow!("some error");
        let app_error: AppError = anyhow_error.into();
        assert!(matches!(app_error, AppError::Other(_)));
        assert_eq!(app_error.exit_code(), ExitCode::from(1));
    }

    #[test]
    fn test_exit_codes_match_expectations() {
        // These tests ensure that the exit codes match the documented values
        assert_eq!(AppError::NoCache.exit_code(), ExitCode::from(2));
        assert_eq!(
            AppError::NotFound("test".to_string()).exit_code(),
            ExitCode::from(3)
        );
        assert_eq!(
            AppError::BuildFailed("test".to_string()).exit_code(),
            ExitCode::from(4)
        );

        assert_eq!(
            AppError::Io(std::io::Error::other("test")).exit_code(),
            ExitCode::from(7)
        );
        assert_eq!(
            AppError::Json(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "test"
            )))
            .exit_code(),
            ExitCode::from(8)
        );
        assert_eq!(
            AppError::Config("test".to_string()).exit_code(),
            ExitCode::from(9)
        );
    }

    #[test]
    fn test_error_display_format() {
        // Test that all error types can be displayed as strings
        let errors = vec![
            AppError::NoCache,
            AppError::NotFound("test::path".to_string()),
            AppError::BuildFailed("build failed".to_string()),
            AppError::Config("config error".to_string()),
        ];

        for error in errors {
            let _ = error.to_string(); // Should not panic
        }
    }
}

/// Custom error types for the application
#[derive(Error, Debug)]
pub enum AppError {
    #[error("No cached index found. Run `cargo doc-query build` first.")]
    NoCache,

    #[error("No items found matching path: {0}")]
    NotFound(String),

    #[error("Failed to build documentation index: {0}")]
    BuildFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    /// Get the appropriate exit code for this error
    pub fn exit_code(&self) -> ExitCode {
        match self {
            AppError::NoCache => ExitCode::from(1),
            AppError::NotFound(_) => ExitCode::from(1),
            AppError::BuildFailed(_) => ExitCode::from(1),
            AppError::Io(_) => ExitCode::from(1),
            AppError::Json(_) => ExitCode::from(1),
            AppError::Config(_) => ExitCode::from(1),
            AppError::Other(_) => ExitCode::from(1),
        }
    }
}
