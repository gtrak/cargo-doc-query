//! Error types and exit codes for cargo-doc-query

use std::process::ExitCode;
use thiserror::Error;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_format() {
        let errors = vec![
            AppError::NoCache,
            AppError::NotFound("test::path".to_string()),
            AppError::BuildFailed("build failed".to_string()),
            AppError::Config("config error".to_string()),
        ];

        for error in errors {
            let _ = error.to_string();
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
