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
    fn test_invalid_query_error_message() {
        let error = AppError::InvalidQuery("invalid path".to_string());
        assert!(error.to_string().contains("invalid path"));
        assert_eq!(error.exit_code(), ExitCode::from(5));
    }

    #[test]
    fn test_cache_error_message() {
        let error = AppError::CacheError("cache storage error".to_string());
        assert!(error.to_string().contains("cache storage error"));
        assert_eq!(error.exit_code(), ExitCode::from(6));
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
            AppError::InvalidQuery("test".to_string()).exit_code(),
            ExitCode::from(5)
        );
        assert_eq!(
            AppError::CacheError("test".to_string()).exit_code(),
            ExitCode::from(6)
        );
        assert_eq!(
            AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test")).exit_code(),
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
            AppError::InvalidQuery("invalid query".to_string()),
            AppError::CacheError("cache error".to_string()),
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

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Cache error: {0}")]
    CacheError(String),

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
            AppError::NoCache => ExitCode::from(2),
            AppError::NotFound(_) => ExitCode::from(3),
            AppError::BuildFailed(_) => ExitCode::from(4),
            AppError::InvalidQuery(_) => ExitCode::from(5),
            AppError::CacheError(_) => ExitCode::from(6),
            AppError::Io(_) => ExitCode::from(7),
            AppError::Json(_) => ExitCode::from(8),
            AppError::Config(_) => ExitCode::from(9),
            AppError::Other(_) => ExitCode::from(1),
        }
    }
}

/// Result type alias using AppError
pub type AppResult<T> = Result<T, AppError>;

/// Exit codes for the application
pub mod exit_codes {
    /// Success
    pub const SUCCESS: i32 = 0;
    /// General error
    pub const GENERAL_ERROR: i32 = 1;
    /// No cache found
    pub const NO_CACHE: i32 = 2;
    /// Query returned no results
    pub const NOT_FOUND: i32 = 3;
    /// Build failed
    pub const BUILD_FAILED: i32 = 4;
    /// Invalid query
    pub const INVALID_QUERY: i32 = 5;
    /// Cache error
    pub const CACHE_ERROR: i32 = 6;
    /// IO error
    pub const IO_ERROR: i32 = 7;
    /// JSON parsing error
    pub const JSON_ERROR: i32 = 8;
    /// Configuration error
    pub const CONFIG_ERROR: i32 = 9;
}
