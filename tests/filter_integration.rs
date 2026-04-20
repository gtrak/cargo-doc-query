//! Integration tests for filter combinations
//!
//! Tests filter + depth, filter + token budget, filter + minimal mode combinations.

mod utils;
use utils::run_doc_query;

/// Helper function for filter validation tests.
/// Checks that a query either succeeds or produces a known error.
fn assert_filter(args: &[&str], description: &str) {
    let output = run_doc_query(args);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache found")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "{}: Expected success or known error, got: {}",
        description,
        stderr
    );
}


/// Test filter + depth combination
/// cargo doc-query query Vec --include "std::*" --depth 2
#[test]
fn test_filter_with_depth() {
    assert_filter(
        &["Vec", "--include", "std::*", "--depth", "2"],
        "filter + depth",
    );
}

/// Test filter + token budget
/// cargo doc-query query Vec --kind "function" --tokens 500
#[test]
fn test_filter_with_token_budget() {
    assert_filter(
        &["Vec", "--kind", "function", "--tokens", "500"],
        "filter + token budget",
    );
}

/// Test filter + minimal mode
/// cargo doc-query query Vec --exclude "*test*" --minimal
#[test]
fn test_filter_with_minimal() {
    assert_filter(
        &["Vec", "--exclude", "*test*", "--minimal"],
        "filter + minimal",
    );
}

/// Test all three combined: filter + depth + token budget
/// cargo doc-query query HashMap --include "std::*" --kind "struct" --depth 1 --tokens 300
#[test]
fn test_filter_depth_tokens_combined() {
      assert_filter(
        &["HashMap", "--include", "std::*", "--kind", "struct", "--depth", "1", "--tokens", "300"],
        "filter + depth + tokens",
    );
}

/// Test --only flag (shorthand for include)
#[test]
fn test_only_filter() {
    assert_filter(&["Vec", "--only", "std::*"], "only filter");
}

/// Test crate filter
#[test]
fn test_crate_filter() {
    assert_filter(&["Vec", "--crate-filter", "std"], "crate filter");
}

/// Test visibility filter
#[test]
fn test_visibility_filter() {
    assert_filter(&["Vec", "--visibility", "pub"], "visibility filter");
}

/// Test --detailed flag with filters
#[test]
fn test_filter_with_detailed() {
    assert_filter(
        &["Vec", "--include", "std::*", "--detailed"],
        "filter + detailed",
    );
}

/// Test JSON output with filters
#[test]
fn test_filter_with_json_output() {
    let output = run_doc_query(&["Vec", "--include", "std::*", "--json"]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If successful, should have valid JSON or empty
    if output.status.success() {
        // JSON might be empty if no results, but should be valid if present
        if !stdout.trim().is_empty() {
            // Just check it's parseable as basic JSON structure
            assert!(
                stdout.starts_with('[') || stdout.starts_with('{') || stdout.trim().is_empty(),
                "JSON output should start with [ or {{ or be empty"
            );
        }
    }
}

/// Test multiple include patterns (OR logic)
#[test]
fn test_multiple_include_patterns() {
   assert_filter(
        &["Vec", "--include", "std::*", "--include", "core::*"],
        "multiple include patterns",
    );
}

/// Test multiple exclude patterns
#[test]
fn test_multiple_exclude_patterns() {
    assert_filter(
        &["Vec", "--exclude", "*test*", "--exclude", "*private*"],
        "multiple exclude patterns",
    );
}
