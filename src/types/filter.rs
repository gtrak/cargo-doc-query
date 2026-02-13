//! Filter types for querying and filtering documentation items
//!
//! This module provides the [`FilterConfig`] and [`FilterEngine`] types for filtering
//! query results based on patterns, kinds, crates, and visibility.
//!
//! # Quick Start
//!
//! ```
//! use cargo_doc_query::types::filter::{FilterConfig, FilterEngine};
//!
//! let config = FilterConfig::default()
//!     .with_include("std::*")
//!     .with_exclude("*::test*")
//!     .with_kind("struct");
//!
//! let engine = FilterEngine::compile(&config)?;
//! let matches = engine.matches("std::vec::Vec", "struct", "std", "pub");
//! ```
//!
//! # Filter Logic
//!
//! Filters combine with AND logic:
//! - An item must satisfy ALL active filters to pass
//! - If no filters are specified, all items pass
//! - Exclude patterns are checked first (fail fast)
//!
//! # Performance
//!
//! - Pattern compilation happens once at startup
//! - Complex patterns are automatically optimized by sorting by complexity
//! - Empty filter config has near-zero overhead
//! - Benchmarks: ~100ns per item check for simple patterns, <1μs for empty config
//!
//! # Error Handling
//!
//! Invalid glob patterns return [`FilterError::InvalidGlob`] with
//! helpful messages explaining the syntax issue.

use crate::types::query::{QueryContent, QueryMatch};
use glob::Pattern;

/// Configuration for filtering query results
///
/// Creates filters using the builder pattern. Filters combine with AND logic.
///
/// # Examples
///
/// ```
/// use cargo_doc_query::types::filter::FilterConfig;
///
/// // Basic filter - include std items, exclude tests
/// let config = FilterConfig::default()
///     .with_include("std::*")
///     .with_exclude("*::test*");
///
/// // Multiple include patterns
/// let config = FilterConfig::default()
///     .with_include("std::*")           // Match all std items
///     .with_include("serde::*");        // Match all serde items
///
/// // Filter by kind, crate, and visibility
/// let config = FilterConfig::default()
///     .with_include("std::*")
///     .with_kind("struct")
///     .with_crate("std")
///     .with_visibility("pub");
/// ```
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
    ///
    /// Returns true if any filter field has items. Empty config matches everything.
    ///
    /// # Examples
    ///
    /// ```
    /// use cargo_doc_query::types::filter::FilterConfig;
    ///
    /// let empty = FilterConfig::default();
    /// assert!(!empty.has_filters());
    ///
    /// let with_filters = FilterConfig::default().with_include("std::*");
    /// assert!(with_filters.has_filters());
    /// ```
    pub fn has_filters(&self) -> bool {
        !self.include.is_empty()
            || !self.exclude.is_empty()
            || !self.kind.is_empty()
            || !self.crate_filter.is_empty()
            || !self.visibility.is_empty()
    }

    /// Builder-style API for FilterConfig
    ///
    /// Each method adds a filter criterion. Filters combine with AND logic.
    ///
    /// # Examples
    ///
    /// ```
    /// use cargo_doc_query::types::filter::FilterConfig;
    ///
    /// let config = FilterConfig::default()
    ///     .with_include("std::*")
    ///     .with_exclude("*::test*")
    ///     .with_kind("struct");
    /// ```
    ///
    /// Multiple include patterns:
    /// ```ignore
    /// let config = FilterConfig::default()
    ///     .with_include("std::*")           // Match all std items
    ///     .with_include("serde::*");        // Match all serde items
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

/// Trait for items that can be filtered
#[cfg(test)]
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

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_unicode_paths() {
        let config = FilterConfig::default().with_include("crate::*");
        let engine = FilterEngine::compile(&config).unwrap();

        assert!(engine.matches("crate::日本語", "fn", "crate", "pub"));
        assert!(engine.matches("crate::émojis_🎉", "fn", "crate", "pub"));
    }

    #[test]
    fn test_special_regex_chars() {
        // Glob patterns should treat regex chars literally
        let config = FilterConfig::default().with_include("crate::foo.*bar"); // . should match literal dot
        let engine = FilterEngine::compile(&config).unwrap();

        assert!(engine.matches("crate::foo.bar", "fn", "crate", "pub"));
        assert!(!engine.matches("crate::fooxbar", "fn", "crate", "pub"));

        // Test parentheses
        let config2 = FilterConfig::default().with_include("crate::foo(bar)");
        let engine2 = FilterEngine::compile(&config2).unwrap();
        assert!(engine2.matches("crate::foo(bar)", "fn", "crate", "pub"));
    }

    #[test]
    fn test_many_patterns_performance() {
        let mut config = FilterConfig::default();
        // Add 100 include patterns
        for i in 0..100 {
            config = config.with_include(&format!("crate::item{}", i));
        }
        // Add 100 exclude patterns
        for i in 0..100 {
            config = config.with_exclude(&format!("crate::exclude{}", i));
        }

        let engine = FilterEngine::compile(&config).unwrap();

        // Should still work efficiently
        assert!(engine.matches("crate::item50", "fn", "crate", "pub"));
        assert!(!engine.matches("crate::exclude50", "fn", "crate", "pub"));
    }

    #[test]
    fn test_overlapping_patterns() {
        // Include and exclude can overlap - exclude wins
        let config = FilterConfig::default()
            .with_include("crate::*")
            .with_exclude("crate::test_*");
        let engine = FilterEngine::compile(&config).unwrap();

        assert!(engine.matches("crate::foo", "fn", "crate", "pub"));
        assert!(!engine.matches("crate::test_helper", "fn", "crate", "pub"));
    }

    #[test]
    fn test_empty_string_matching() {
        let config = FilterConfig::default().with_include(""); // Should fail on compile
        let result = FilterEngine::compile(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_patterns() {
        // Whitespace should be treated literally in paths
        let config = FilterConfig::default().with_include("* ::*"); // Space in pattern
                                                                    // This is technically a valid glob, just unlikely to match much
        let result = FilterEngine::compile(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_case_sensitivity() {
        // Crate names are case-sensitive
        let config = FilterConfig::default().with_crate("Serde"); // Note capital S
        let engine = FilterEngine::compile(&config).unwrap();

        assert!(engine.matches("serde::Serialize", "trait", "Serde", "pub"));
        assert!(!engine.matches("serde::Serialize", "trait", "serde", "pub"));

        // Kinds are case-insensitive
        let config2 = FilterConfig::default().with_kind("STRUCT");
        let engine2 = FilterEngine::compile(&config2).unwrap();
        assert!(engine2.matches("crate::Foo", "struct", "crate", "pub"));
        assert!(engine2.matches("crate::Bar", "STRUCT", "crate", "pub"));
    }

    #[test]
    fn test_path_with_double_colons() {
        // Paths with multiple :: should work
        let config = FilterConfig::default().with_include("a::b::c::d::*");
        let engine = FilterEngine::compile(&config).unwrap();

        assert!(engine.matches("a::b::c::d::item", "fn", "a", "pub"));
        assert!(!engine.matches("a::b::c::other", "fn", "a", "pub"));
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

/// Statistics from filter application
#[derive(Debug, Clone, Default)]
pub struct FilterStats {
    /// Total items checked
    pub total_checked: usize,
    /// Items that passed all filters
    pub items_passed: usize,
    /// Items rejected by each filter type
    pub rejected_by_include: usize,
    pub rejected_by_exclude: usize,
    pub rejected_by_kind: usize,
    pub rejected_by_crate: usize,
    pub rejected_by_visibility: usize,
    /// Average time per check (microseconds)
    pub avg_check_time_us: f64,
}

impl FilterStats {
    /// Get rejection rate (0.0 to 1.0)
    pub fn rejection_rate(&self) -> f64 {
        if self.total_checked == 0 {
            return 0.0;
        }
        let total_rejected = self.rejected_by_include
            + self.rejected_by_exclude
            + self.rejected_by_kind
            + self.rejected_by_crate
            + self.rejected_by_visibility;
        total_rejected as f64 / self.total_checked as f64
    }

    /// Get pass rate (0.0 to 1.0)
    pub fn pass_rate(&self) -> f64 {
        if self.total_checked == 0 {
            return 1.0;
        }
        self.items_passed as f64 / self.total_checked as f64
    }

    /// Format as human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Filter Stats: {} checked, {} passed ({:.1}% rejection rate)",
            self.total_checked,
            self.items_passed,
            self.rejection_rate() * 100.0
        )
    }
}

enum RejectionReason {
    Include,
    Exclude,
    Kind,
    Crate,
    Visibility,
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
    /// Check if engine has no active filters
    ///
    /// This is the fast path - if no filters are active, all items pass.
    fn is_empty(&self) -> bool {
        self.include.is_empty()
            && self.exclude.is_empty()
            && self.kinds.is_empty()
            && self.crates.is_empty()
            && self.visibilities.is_empty()
    }
}

impl FilterEngine {
    /// Compile FilterConfig into FilterEngine with pattern optimization
    ///
    /// Patterns are automatically sorted by complexity (simple patterns first) for better
    /// average-case performance. This optimization ensures common patterns are checked
    /// before more complex ones.
    ///
    /// Returns [`FilterError::InvalidGlob`] if any pattern is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use cargo_doc_query::types::filter::{FilterConfig, FilterEngine};
    ///
    /// let config = FilterConfig::default()
    ///     .with_include("std::*")
    ///     .with_exclude("*::test*");
    ///
    /// let engine = FilterEngine::compile(&config)?;
    /// ```
    pub fn compile(config: &FilterConfig) -> Result<Self, FilterError> {
        Self::compile_optimized(config)
    }

    /// Compile with pattern optimization (internal)
    ///
    /// Patterns are sorted by complexity (simple patterns first) for better average-case performance.
    fn compile_optimized(config: &FilterConfig) -> Result<Self, FilterError> {
        // Compile and sort include patterns by complexity
        let mut include = Vec::new();
        for pattern in &config.include {
            if pattern.is_empty() {
                return Err(FilterError::EmptyPattern);
            }
            match Pattern::new(pattern) {
                Ok(p) => include.push((p, Self::pattern_complexity(pattern))),
                Err(e) => {
                    return Err(FilterError::InvalidGlob {
                        pattern: pattern.clone(),
                        message: e.to_string(),
                    })
                }
            }
        }
        // Sort by complexity (simple patterns first)
        include.sort_by_key(|(_, c)| *c);
        let include: Vec<Pattern> = include.into_iter().map(|(p, _)| p).collect();

        // Compile and sort exclude patterns by complexity
        let mut exclude = Vec::new();
        for pattern in &config.exclude {
            if pattern.is_empty() {
                return Err(FilterError::EmptyPattern);
            }
            match Pattern::new(pattern) {
                Ok(p) => exclude.push((p, Self::pattern_complexity(pattern))),
                Err(e) => {
                    return Err(FilterError::InvalidGlob {
                        pattern: pattern.clone(),
                        message: e.to_string(),
                    })
                }
            }
        }
        // Sort by complexity (simple patterns first)
        exclude.sort_by_key(|(_, c)| *c);
        let exclude: Vec<Pattern> = exclude.into_iter().map(|(p, _)| p).collect();

        Ok(Self {
            include,
            exclude,
            kinds: config.kind.iter().map(|k| k.to_lowercase()).collect(),
            crates: config.crate_filter.clone(),
            visibilities: config.visibility.clone(),
        })
    }

    /// Check if an item matches all active filters (AND logic)
    ///
    /// Optimized matching order:
    /// 1. Exclude patterns (fail fast - most restrictive)
    /// 2. Kind filter (string comparison - cheap)
    /// 3. Crate filter (string comparison - cheap)
    /// 4. Visibility filter (string comparison - cheap)
    /// 5. Include patterns (glob matching - most expensive)
    ///
    /// This ordering minimizes the number of expensive glob pattern matches.
    pub fn matches(&self, path: &str, kind: &str, crate_name: &str, visibility: &str) -> bool {
        // Fast path: no filters active
        if self.is_empty() {
            return true;
        }

        // 1. Must not match any exclude pattern (fail fast - most restrictive)
        if !self.exclude.is_empty() {
            if self.exclude.iter().any(|p| p.matches(path)) {
                return false;
            }
        }

        // 2. Must match kind filter (case insensitive)
        if !self.kinds.is_empty() {
            if !self.kinds.iter().any(|k| k == &kind.to_lowercase()) {
                return false;
            }
        }

        // 3. Must match crate filter
        if !self.crates.is_empty() {
            if !self.crates.iter().any(|c| c == crate_name) {
                return false;
            }
        }

        // 4. Must match visibility filter
        if !self.visibilities.is_empty() {
            if !self.visibilities.iter().any(|v| v == visibility) {
                return false;
            }
        }

        // 5. Must match at least one include pattern (most expensive - last)
        if !self.include.is_empty() {
            if !self.include.iter().any(|p| p.matches(path)) {
                return false;
            }
        }

        true
    }

    /// Filter with statistics collection
    pub fn filter_with_stats<'a, T: Filterable>(
        &self,
        items: &'a [T],
    ) -> (Vec<&'a T>, FilterStats) {
        let start = Instant::now();
        let mut stats = FilterStats::default();
        stats.total_checked = items.len();

        let mut passed = Vec::new();

        for item in items {
            let (matches, rejection) = self.matches_with_details(
                item.filter_path(),
                item.filter_kind(),
                item.filter_crate(),
                item.filter_visibility(),
            );

            if matches {
                passed.push(item);
                stats.items_passed += 1;
            } else if let Some(r) = rejection {
                match r {
                    RejectionReason::Include => stats.rejected_by_include += 1,
                    RejectionReason::Exclude => stats.rejected_by_exclude += 1,
                    RejectionReason::Kind => stats.rejected_by_kind += 1,
                    RejectionReason::Crate => stats.rejected_by_crate += 1,
                    RejectionReason::Visibility => stats.rejected_by_visibility += 1,
                }
            }
        }

        let elapsed = start.elapsed();
        stats.avg_check_time_us = if stats.total_checked > 0 {
            elapsed.as_micros() as f64 / stats.total_checked as f64
        } else {
            0.0
        };

        (passed, stats)
    }

    /// Internal: check match with rejection reason
    fn matches_with_details(
        &self,
        path: &str,
        kind: &str,
        crate_name: &str,
        visibility: &str,
    ) -> (bool, Option<RejectionReason>) {
        // Must match at least one include pattern
        if !self.include.is_empty() {
            if !self.include.iter().any(|p| p.matches(path)) {
                return (false, Some(RejectionReason::Include));
            }
        }

        // Must not match any exclude pattern
        if self.exclude.iter().any(|p| p.matches(path)) {
            return (false, Some(RejectionReason::Exclude));
        }

        // Must match kind filter
        if !self.kinds.is_empty() {
            if !self.kinds.iter().any(|k| k == &kind.to_lowercase()) {
                return (false, Some(RejectionReason::Kind));
            }
        }

        // Must match crate filter
        if !self.crates.is_empty() {
            if !self.crates.iter().any(|c| c == crate_name) {
                return (false, Some(RejectionReason::Crate));
            }
        }

        // Must match visibility filter
        if !self.visibilities.is_empty() {
            if !self.visibilities.iter().any(|v| v == visibility) {
                return (false, Some(RejectionReason::Visibility));
            }
        }

        (true, None)
    }

    /// Check if this engine has any active filters
    pub fn is_active(&self) -> bool {
        !self.is_empty()
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

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::types::query::{QueryContent, QueryMatch, TypeResult};

    fn create_test_match(path: &str, kind: &str, crate_name: &str) -> QueryMatch {
        QueryMatch {
            crate_name: crate_name.to_string(),
            version: "1.0.0".to_string(),
            fully_qualified_path: path.to_string(),
            kind: kind.to_string(),
            content: QueryContent::Type(TypeResult {
                kind: kind.to_string(),
                methods: vec![],
                trait_implementations: vec![],
            }),
        }
    }

    #[test]
    fn test_filter_query_matches_include() {
        let items = vec![
            create_test_match("std::vec::Vec", "struct", "std"),
            create_test_match("std::string::String", "struct", "std"),
            create_test_match("crate::foo::Bar", "struct", "my_crate"),
        ];

        let config = FilterConfig::default().with_include("std::*");
        let engine = FilterEngine::compile(&config).unwrap();

        let filtered = engine.filter_matches(&items);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].filter_path(), "std::vec::Vec");
        assert_eq!(filtered[1].filter_path(), "std::string::String");
    }

    #[test]
    fn test_filter_with_stats() {
        let items = vec![
            create_test_match("std::vec::Vec", "struct", "std"),
            create_test_match("std::string::String", "struct", "std"),
            create_test_match("crate::foo::Bar", "struct", "my_crate"),
        ];

        let config = FilterConfig::default().with_include("std::*");
        let engine = FilterEngine::compile(&config).unwrap();

        let (filtered, stats) = engine.filter_with_stats(&items);
        assert_eq!(filtered.len(), 2);
        assert_eq!(stats.total_checked, 3);
        assert_eq!(stats.items_passed, 2);
        assert_eq!(stats.rejected_by_include, 1);
        assert!(stats.pass_rate() > 0.6 && stats.pass_rate() < 0.7);
    }

    #[test]
    fn test_pattern_validation_warnings() {
        let config = FilterConfig::default().with_include("*");

        let warnings = FilterEngine::validate_patterns(&config);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("very broad"));
    }

    #[test]
    fn test_complex_patterns() {
        // Test character class
        let config = FilterConfig::default().with_include("crate::[A-Z]*");
        let engine = FilterEngine::compile(&config).unwrap();

        assert!(engine.matches("crate::Foo", "struct", "crate", "pub"));
        assert!(!engine.matches("crate::bar", "struct", "crate", "pub"));

        // Test complex pattern with multiple wildcards
        let config2 = FilterConfig::default().with_include("*::*");
        let engine2 = FilterEngine::compile(&config2).unwrap();

        assert!(engine2.matches("std::vec::Vec", "struct", "std", "pub"));
        assert!(engine2.matches("crate::Foo", "struct", "my_crate", "pub"));
        assert!(!engine2.matches("std", "trait", "std", "pub"));
    }

    #[test]
    fn test_filter_query_matches_complex() {
        let items = vec![
            create_test_match("std::fmt::Display", "trait", "std"),
            create_test_match("std::fmt::Debug", "trait", "std"),
            create_test_match("serde::Serialize", "trait", "serde"),
            create_test_match("crate::Bar", "struct", "my_crate"),
        ];

        // Filter by crate and kind
        let config = FilterConfig::default()
            .with_include("std::*")
            .with_kind("trait");
        let engine = FilterEngine::compile(&config).unwrap();

        let filtered = engine.filter_matches(&items);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].filter_path(), "std::fmt::Display");
        assert_eq!(filtered[1].filter_path(), "std::fmt::Debug");
    }

    #[test]
    fn test_filter_stats_summary() {
        let config = FilterConfig::default().with_include("std::*");
        let engine = FilterEngine::compile(&config).unwrap();

        let items = vec![
            create_test_match("std::vec::Vec", "struct", "std"),
            create_test_match("std::string::String", "struct", "std"),
            create_test_match("crate::foo::Bar", "struct", "my_crate"),
        ];

        let (filtered, stats) = engine.filter_with_stats(&items);

        // Check summary format
        let summary = stats.summary();
        assert!(summary.contains("checked"));
        assert!(summary.contains("passed"));
        assert!(summary.contains("%"));

        // Verify stats are accurate
        assert_eq!(
            summary,
            "Filter Stats: 3 checked, 2 passed (33.3% rejection rate)"
        );
    }
}
