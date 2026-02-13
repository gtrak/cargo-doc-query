//! Filter types for querying and filtering documentation items
//!
//! This module provides the FilterConfig and FilterEngine types for filtering
//! query results based on patterns, kinds, crates, and visibility.

/// Configuration for filtering query results
#[derive(Debug, Clone, Default)]
pub struct FilterConfig {
    /// Include patterns (glob) - items must match at least one
    pub include: Vec<String>,
    /// Exclude patterns (glob) - items must not match any
    pub exclude: Vec<String>,
    /// Filter by item kind (struct, enum, trait, function, etc.)
    pub kind: Vec<String>,
    /// Filter by crate name
    pub crate_filter: Vec<String>,
    /// Filter by visibility (pub, pub(crate), pub(super), pub(in path))
    pub visibility: Vec<String>,
}

impl FilterConfig {
    /// Check if any filters are configured
    pub fn has_filters(&self) -> bool {
        !self.include.is_empty()
            || !self.exclude.is_empty()
            || !self.kind.is_empty()
            || !self.crate_filter.is_empty()
            || !self.visibility.is_empty()
    }

    /// Builder-style API for FilterConfig
    ///
    /// Example:
    /// ```
    /// use cargo_doc_query::types::filter::FilterConfig;
    ///
    /// let config = FilterConfig::default()
    ///     .with_include("std::*")
    ///     .with_exclude("*::test*")
    ///     .with_kind("struct");
    /// ```
    pub fn with_include(mut self, pattern: impl Into<String>) -> Self {
        self.include.push(pattern.into());
        self
    }

    pub fn with_exclude(mut self, pattern: impl Into<String>) -> Self {
        self.exclude.push(pattern.into());
        self
    }

    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind.push(kind.into());
        self
    }

    pub fn with_crate(mut self, crate_name: impl Into<String>) -> Self {
        self.crate_filter.push(crate_name.into());
        self
    }

    pub fn with_visibility(mut self, vis: impl Into<String>) -> Self {
        self.visibility.push(vis.into());
        self
    }
}

/// Errors that can occur during filter pattern compilation
#[derive(Error, Debug, Clone)]
pub enum FilterError {
    #[error("Invalid glob pattern '{pattern}': {message}")]
    InvalidGlob { pattern: String, message: String },
    #[error("Empty pattern provided")]
    EmptyPattern,
    #[error("Conflicting filters: include and exclude patterns match the same item")]
    ConflictingFilters { item: String },
}

impl From<glob::PatternError> for FilterError {
    fn from(err: glob::PatternError) -> Self {
        Self::InvalidGlob {
            pattern: err
                .pos()
                .map(|p| format!("at position {}", p))
                .unwrap_or_default(),
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_config_defaults() {
        let config = FilterConfig::default();
        assert_eq!(config.include, vec![]);
        assert_eq!(config.exclude, vec![]);
        assert_eq!(config.kind, vec![]);
        assert_eq!(config.crate_filter, vec![]);
        assert_eq!(config.visibility, vec![]);
    }

    #[test]
    fn test_filter_config_builder() {
        let config = FilterConfig::default()
            .with_include("std::*")
            .with_exclude("*Test*")
            .with_kind("function")
            .with_crate("serde")
            .with_visibility("pub(crate)");

        assert_eq!(config.include, vec!["std::*"]);
        assert_eq!(config.exclude, vec!["*Test*"]);
        assert_eq!(config.kind, vec!["function"]);
        assert_eq!(config.crate_filter, vec!["serde"]);
        assert_eq!(config.visibility, vec!["pub(crate)"]);
    }

    #[test]
    fn test_filter_config_has_filters() {
        let config = FilterConfig::default();
        assert!(!config.has_filters());

        let config_with_filters = FilterConfig::default().with_include("std::*");
        assert!(config_with_filters.has_filters());
    }
}
