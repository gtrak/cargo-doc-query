//! Filter types for querying and filtering documentation items
//!
//! This module provides the FilterConfig and FilterEngine types for filtering
//! query results based on patterns, kinds, crates, and visibility.

use glob::Pattern;

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

/// Compiled filter engine for efficient pattern matching
#[derive(Debug, Clone)]
pub struct FilterEngine {
    /// Compiled include patterns
    include: Vec<Pattern>,
    /// Compiled exclude patterns
    exclude: Vec<Pattern>,
    /// Kind filters (case-insensitive matching)
    kinds: Vec<String>,
    /// Crate name filters
    crates: Vec<String>,
    /// Visibility filters
    visibilities: Vec<String>,
}

impl FilterEngine {
    /// Compile FilterConfig into FilterEngine
    ///
    /// Returns FilterError if any pattern is invalid
    pub fn compile(config: &FilterConfig) -> Result<Self, FilterError> {
        let mut include = Vec::new();
        for pattern in &config.include {
            if pattern.is_empty() {
                return Err(FilterError::EmptyPattern);
            }
            match Pattern::new(pattern) {
                Ok(p) => include.push(p),
                Err(e) => {
                    return Err(FilterError::InvalidGlob {
                        pattern: pattern.clone(),
                        message: e.to_string(),
                    })
                }
            }
        }

        let mut exclude = Vec::new();
        for pattern in &config.exclude {
            if pattern.is_empty() {
                return Err(FilterError::EmptyPattern);
            }
            match Pattern::new(pattern) {
                Ok(p) => exclude.push(p),
                Err(e) => {
                    return Err(FilterError::InvalidGlob {
                        pattern: pattern.clone(),
                        message: e.to_string(),
                    })
                }
            }
        }

        Ok(Self {
            include,
            exclude,
            kinds: config.kind.iter().map(|k| k.to_lowercase()).collect(),
            crates: config.crate_filter.clone(),
            visibilities: config.visibility.clone(),
        })
    }

    /// Check if an item matches all active filters (AND logic)
    pub fn matches(&self, path: &str, kind: &str, crate_name: &str, visibility: &str) -> bool {
        // Must match at least one include pattern (if any specified)
        if !self.include.is_empty() {
            if !self.include.iter().any(|p| p.matches(path)) {
                return false;
            }
        }

        // Must not match any exclude pattern
        if self.exclude.iter().any(|p| p.matches(path)) {
            return false;
        }

        // Must match kind filter (if specified) - case insensitive
        if !self.kinds.is_empty() {
            if !self.kinds.iter().any(|k| k == &kind.to_lowercase()) {
                return false;
            }
        }

        // Must match crate filter (if specified)
        if !self.crates.is_empty() {
            if !self.crates.iter().any(|c| c == crate_name) {
                return false;
            }
        }

        // Must match visibility filter (if specified)
        if !self.visibilities.is_empty() {
            if !self.visibilities.iter().any(|v| v == visibility) {
                return false;
            }
        }

        true
    }

    /// Check if this engine has any active filters
    pub fn is_active(&self) -> bool {
        !self.include.is_empty()
            || !self.exclude.is_empty()
            || !self.kinds.is_empty()
            || !self.crates.is_empty()
            || !self.visibilities.is_empty()
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
