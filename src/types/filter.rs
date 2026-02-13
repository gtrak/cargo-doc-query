//! Filter types for querying and filtering documentation items
//!
//! This module provides the FilterConfig and FilterEngine types for filtering
//! query results based on patterns, kinds, crates, and visibility.

use crate::types::query::{QueryContent, QueryMatch};
use glob::Pattern;
use thiserror::Error;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_filter_matches_all() {
        let config = FilterConfig::default();
        let engine = FilterEngine::compile(&config).unwrap();
        assert!(engine.matches("any::path", "struct", "my_crate", "pub"));
    }

    #[test]
    fn test_include_pattern_matching() {
        let config = FilterConfig::default().with_include("std::*");
        let engine = FilterEngine::compile(&config).unwrap();
        assert!(engine.matches("std::vec::Vec", "struct", "std", "pub"));
        assert!(!engine.matches("crate::foo::Bar", "struct", "my_crate", "pub"));
    }

    #[test]
    fn test_exclude_pattern_filtering() {
        let config = FilterConfig::default().with_exclude("*Test*");
        let engine = FilterEngine::compile(&config).unwrap();
        assert!(!engine.matches("my_crate::TestStruct", "struct", "my_crate", "pub"));
        assert!(engine.matches("my_crate::RealStruct", "struct", "my_crate", "pub"));
    }

    #[test]
    fn test_include_and_exclude_combined() {
        let config = FilterConfig::default()
            .with_include("std::*")
            .with_exclude("*::test*");
        let engine = FilterEngine::compile(&config).unwrap();
        assert!(engine.matches("std::vec::Vec", "struct", "std", "pub"));
        assert!(!engine.matches("std::test::Test", "struct", "std", "pub"));
        assert!(!engine.matches("crate::foo", "struct", "my_crate", "pub"));
    }

    #[test]
    fn test_kind_filtering() {
        let config = FilterConfig::default().with_kind("function");
        let engine = FilterEngine::compile(&config).unwrap();
        assert!(engine.matches("crate::foo", "function", "my_crate", "pub"));
        assert!(!engine.matches("crate::foo", "struct", "my_crate", "pub"));
    }

    #[test]
    fn test_crate_filtering() {
        let config = FilterConfig::default().with_crate("serde");
        let engine = FilterEngine::compile(&config).unwrap();
        assert!(engine.matches("serde::Serialize", "trait", "serde", "pub"));
        assert!(!engine.matches("std::vec::Vec", "struct", "std", "pub"));
    }

    #[test]
    fn test_visibility_filtering() {
        let config = FilterConfig::default().with_visibility("pub(crate)");
        let engine = FilterEngine::compile(&config).unwrap();
        assert!(engine.matches("crate::foo", "fn", "my_crate", "pub(crate)"));
        assert!(!engine.matches("crate::bar", "fn", "my_crate", "pub"));
    }

    #[test]
    fn test_invalid_glob_pattern_error() {
        let config = FilterConfig::default().with_include("[invalid");
        let result = FilterEngine::compile(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid glob pattern"));
    }

    #[test]
    fn test_empty_pattern_error() {
        let config = FilterConfig::default().with_include("");
        let result = FilterEngine::compile(&config);
        assert!(matches!(result, Err(FilterError::EmptyPattern)));
    }

    #[test]
    fn test_kind_case_insensitive() {
        let config = FilterConfig::default().with_kind("STRUCT");
        let engine = FilterEngine::compile(&config).unwrap();
        assert!(engine.matches("crate::Foo", "struct", "my_crate", "pub"));
        assert!(engine.matches("crate::Bar", "STRUCT", "my_crate", "pub"));
    }

    #[test]
    fn test_engine_is_active() {
        let config = FilterConfig::default();
        assert!(!FilterEngine::compile(&config).unwrap().is_active());

        let config_with_filters = FilterConfig::default().with_include("std::*");
        assert!(FilterEngine::compile(&config_with_filters)
            .unwrap()
            .is_active());
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
            pattern: String::new(),
            message: err.to_string(),
        }
    }
}

/// Trait for items that can be filtered
pub trait Filterable {
    /// Get the fully qualified path of the item
    fn filter_path(&self) -> &str;
    /// Get the item kind (struct, enum, trait, function, etc.)
    fn filter_kind(&self) -> &str;
    /// Get the crate name
    fn filter_crate(&self) -> &str;
    /// Get the visibility as a string
    fn filter_visibility(&self) -> &str;
}

impl Filterable for QueryMatch {
    fn filter_path(&self) -> &str {
        &self.fully_qualified_path
    }

    fn filter_kind(&self) -> &str {
        &self.kind
    }

    fn filter_crate(&self) -> &str {
        &self.crate_name
    }

    fn filter_visibility(&self) -> &str {
        // Try to get visibility from content
        match &self.content {
            QueryContent::Type(t) => {
                // Check if any method has visibility info
                t.methods
                    .first()
                    .map(|m| m.visibility.as_str())
                    .unwrap_or("pub")
            }
            QueryContent::Trait(tr) => tr
                .methods
                .first()
                .map(|m| m.visibility.as_str())
                .unwrap_or("pub"),
            QueryContent::Module(_) => "pub",
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

    /// Filter a slice of QueryMatch items
    ///
    /// Returns only items that match all active filters
    pub fn filter_matches<'a, T: Filterable>(&self, items: &'a [T]) -> Vec<&'a T> {
        items
            .iter()
            .filter(|item| {
                self.matches(
                    item.filter_path(),
                    item.filter_kind(),
                    item.filter_crate(),
                    item.filter_visibility(),
                )
            })
            .collect()
    }

    /// Filter and clone matches (for owned collections)
    pub fn filter_matches_owned<T: Filterable + Clone>(&self, items: &[T]) -> Vec<T> {
        items
            .iter()
            .filter(|item| {
                self.matches(
                    item.filter_path(),
                    item.filter_kind(),
                    item.filter_crate(),
                    item.filter_visibility(),
                )
            })
            .cloned()
            .collect()
    }

    /// Check if this engine has any active filters
    pub fn is_active(&self) -> bool {
        !self.include.is_empty()
            || !self.exclude.is_empty()
            || !self.kinds.is_empty()
            || !self.crates.is_empty()
            || !self.visibilities.is_empty()
    }

    /// Validate that patterns are reasonable and not overly broad
    ///
    /// Returns warnings for patterns that might match too many items
    pub fn validate_patterns(config: &FilterConfig) -> Vec<String> {
        let mut warnings = Vec::new();

        // Warn about overly broad patterns
        for pattern in &config.include {
            if pattern == "*" || pattern == "**" {
                warnings.push(format!(
                    "Include pattern '{}' is very broad and may match all items. \
                    Consider using a more specific pattern like 'crate::*'.",
                    pattern
                ));
            }
        }

        // Check for duplicate patterns
        for (i, pattern) in config.include.iter().enumerate() {
            for other in config.include.iter().skip(i + 1) {
                if pattern == other {
                    warnings.push(format!("Duplicate include pattern: '{}'", pattern));
                }
            }
        }

        warnings
    }

    /// Get help text for glob syntax
    pub fn glob_syntax_help() -> &'static str {
        r#"Glob Pattern Syntax:

*       Matches any sequence of characters (except path separator)
?       Matches any single character
**     Matches any sequence of characters including path separators
[...]  Matches any character in the bracket
[!...] Matches any character not in the bracket

Examples:
  'std::*'              - Match all items in std crate
  'std::vec::*'         - Match all items in std::vec module
  '*::test*'            - Match any item with 'test' in name
  '**::Display'         - Match Display trait anywhere
  'crate::[A-Z]*'       - Match items starting with capital letter
"#
    }

    /// Estimate pattern complexity (higher = more items likely to match)
    ///
    /// Used for performance optimization - simple patterns are faster
    pub fn pattern_complexity(pattern: &str) -> u32 {
        let mut complexity = 0u32;
        for c in pattern.chars() {
            match c {
                '*' => complexity += 10,
                '?' => complexity += 5,
                '[' => complexity += 15,
                _ => complexity += 1,
            }
        }
        complexity
    }
}
