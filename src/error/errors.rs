//! Error types and exit codes for cargo-doc-query

use std::process::ExitCode;
use thiserror::Error;

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
