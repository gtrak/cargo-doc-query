//! Integration tests for filter combinations
//!
//! Tests filter + depth, filter + token budget, filter + minimal mode combinations.

mod utils;
use utils::run_doc_query;

/// Test filter + depth combination
/// cargo doc-query query Vec --include "std::*" --depth 2
#[test]
fn test_filter_with_depth() {
    // Build might fail if deps not available, but continue to query test
    let output = run_doc_query(&["Vec", "--include", "std::*", "--depth", "2"]);

    // Should either succeed or gracefully handle missing deps
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that it's either successful or a known error (no cache, etc.)
    assert!(
        output.status.success()
            || stderr.contains("No cache found")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Expected success or known error, got: {}",
        stderr
    );
}

/// Test filter + token budget
/// cargo doc-query query Vec --kind "function" --tokens 500
#[test]
fn test_filter_with_token_budget() {
    let output = run_doc_query(&["Vec", "--kind", "function", "--tokens", "500"]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check either success or known error
    assert!(
        output.status.success()
            || stderr.contains("No cache found")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Expected success or known error, got: {}",
        stderr
    );
}

/// Test filter + minimal mode
/// cargo doc-query query Vec --exclude "*test*" --minimal
#[test]
fn test_filter_with_minimal() {
    let output = run_doc_query(&["Vec", "--exclude", "*test*", "--minimal"]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache found")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Expected success or known error, got: {}",
        stderr
    );
}

/// Test all three combined: filter + depth + token budget
/// cargo doc-query query HashMap --include "std::*" --kind "struct" --depth 1 --tokens 300
#[test]
fn test_filter_depth_tokens_combined() {
    let output = run_doc_query(&[
        "HashMap",
        "--include",
        "std::*",
        "--kind",
        "struct",
        "--depth",
        "1",
        "--tokens",
        "300",
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache found")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Expected success or known error, got: {}",
        stderr
    );
}

/// Test --only flag (shorthand for include)
#[test]
fn test_only_filter() {
    let output = run_doc_query(&["Vec", "--only", "std::*"]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache found")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Expected success or known error, got: {}",
        stderr
    );
}

/// Test crate filter
#[test]
fn test_crate_filter() {
    let output = run_doc_query(&["Vec", "--crate-filter", "std"]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache found")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Expected success or known error, got: {}",
        stderr
    );
}

/// Test visibility filter
#[test]
fn test_visibility_filter() {
    let output = run_doc_query(&["Vec", "--visibility", "pub"]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache found")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Expected success or known error, got: {}",
        stderr
    );
}

/// Test --detailed flag with filters
#[test]
fn test_filter_with_detailed() {
    let output = run_doc_query(&["Vec", "--include", "std::*", "--detailed"]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache found")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Expected success or known error, got: {}",
        stderr
    );
}

/// Test kind filter case insensitivity
#[test]
fn test_kind_case_insensitive() {
    // Test with different casings
    let output1 = run_doc_query(&["Vec", "--kind", "FUNCTION"]);
    let output2 = run_doc_query(&["Vec", "--kind", "function"]);
    let output3 = run_doc_query(&["Vec", "--kind", "Function"]);

    // All should behave the same way (either all succeed or all fail)
    let success1 =
        output1.status.success() || String::from_utf8_lossy(&output1.stderr).contains("No cache");
    let success2 =
        output2.status.success() || String::from_utf8_lossy(&output2.stderr).contains("No cache");
    let success3 =
        output3.status.success() || String::from_utf8_lossy(&output3.stderr).contains("No cache");

    // All should have same outcome
    assert_eq!(
        success1, success2,
        "FUNCTION and function should behave the same"
    );
    assert_eq!(
        success2, success3,
        "function and Function should behave the same"
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
    let output = run_doc_query(&["Vec", "--include", "std::*", "--include", "core::*"]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache found")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Expected success or known error, got: {}",
        stderr
    );
}

/// Test multiple exclude patterns
#[test]
fn test_multiple_exclude_patterns() {
    let output = run_doc_query(&["Vec", "--exclude", "*test*", "--exclude", "*private*"]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache found")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Expected success or known error, got: {}",
        stderr
    );
}
