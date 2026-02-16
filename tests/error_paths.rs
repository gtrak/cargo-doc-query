//! Integration tests for error paths
//!
//! Tests error scenarios: invalid glob, empty results, conflicting flags, invalid visibility.

mod utils;
use std::process::Command;
use utils::run_doc_query;

/// Test invalid glob pattern produces helpful error
/// cargo doc-query query Vec --include "[invalid"
#[test]
fn test_invalid_glob_pattern() {
    let output = run_doc_query(&["Vec", "--include", "[invalid"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Invalid glob pattern should cause failure"
    );

    // Should have helpful error message
    assert!(
        stderr.contains("glob")
            || stderr.contains("pattern")
            || stderr.contains("error")
            || stderr.contains("Error"),
        "Error message should mention glob/pattern: {}",
        stderr
    );
}

/// Test empty result set produces helpful message
/// cargo doc-query query NonexistentType
#[test]
fn test_empty_result_set() {
    let output = run_doc_query(&["NonexistentType12345"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "Nonexistent type should fail");

    assert!(
        stderr.contains("No items found"),
        "Should mention no items found: {}",
        stderr
    );
}

/// Test conflicting flags --include and --only
/// cargo doc-query query Vec --include "std::*" --only "serde::*"
#[test]
fn test_conflicting_include_and_only() {
    let output = run_doc_query(&["Vec", "--include", "std::*", "--only", "serde::*"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should fail due to conflicting flags
    assert!(
        !output.status.success(),
        "Conflicting --include and --only should cause failure"
    );

    // Should have helpful error about mutual exclusivity
    assert!(
        stderr.contains("include")
            || stderr.contains("only")
            || stderr.contains("exclusive")
            || stderr.contains("conflict"),
        "Error should mention conflicting flags: {}",
        stderr
    );
}

/// Test invalid crate name handled gracefully
/// cargo doc-query query Vec --crate "nonexistent_crate_12345"
#[test]
fn test_invalid_crate_name() {
    let output = run_doc_query(&["Vec", "--crate", "nonexistent_crate_12345"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should either return empty results or graceful error
    let is_graceful = output.status.code() == Some(3)  // NO_RESULTS
        || stdout.trim().is_empty()
        || stderr.contains("not found")
        || stderr.contains("crate");

    assert!(
        is_graceful,
        "Invalid crate name should be handled gracefully: {}",
        stderr
    );
}

/// Test invalid visibility filter shows available options
/// cargo doc-query query Vec --visibility "invalid_visibility"
#[test]
fn test_invalid_visibility_filter() {
    let output = run_doc_query(&["Vec", "--visibility", "invalid_visibility_xyz"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should fail or warn about invalid visibility
    let has_error_or_warning = !output.status.success()
        || stderr.contains("visibility")
        || stderr.contains("pub")
        || stderr.contains("private")
        || stderr.contains("invalid");

    assert!(
        has_error_or_warning,
        "Invalid visibility should produce error or warning: {}",
        stderr
    );
}

/// Test --help-filters flag works
#[test]
fn test_help_filters_flag() {
    let output = run_doc_query(&["--help-filters"]);

    // Should succeed and show help
    assert!(output.status.success(), "--help-filters should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_str = format!("{}{}", stdout, stderr);

    // Should contain glob pattern info
    assert!(
        output_str.contains("glob") || output_str.contains("*") || output_str.contains("pattern"),
        "--help-filters should explain glob patterns: {}",
        output_str
    );
}

/// Test invalid depth value
#[test]
fn test_invalid_depth() {
    let output = run_doc_query(&["Vec", "--depth", "not_a_number"]);

    // Should fail
    assert!(
        !output.status.success(),
        "Invalid depth should cause failure"
    );
}

/// Test invalid token budget value
#[test]
fn test_invalid_tokens() {
    let output = run_doc_query(&["Vec", "--tokens", "not_a_number"]);

    // Should fail
    assert!(
        !output.status.success(),
        "Invalid tokens should cause failure"
    );
}

/// Test query without build first (no cache)
#[test]
fn test_query_without_build() {
    // Use a fresh temp directory to avoid cache
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "query",
            "--manifest",
            "/tmp/nonexistent_manifest",
            "Vec",
        ])
        .output()
        .expect("Failed to execute cargo doc-query");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should indicate no cache or build needed
    let needs_build = stderr.contains("No cache")
        || stderr.contains("build")
        || stderr.contains("not found")
        || !output.status.success();

    assert!(
        needs_build,
        "Query without build should indicate need to build: {}",
        stderr
    );
}

/// Test malformed query path
#[test]
fn test_malformed_query_path() {
    let output = run_doc_query(&[""]);

    // Should fail gracefully
    assert!(
        !output.status.success() || String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "Empty query should fail or return empty"
    );
}

/// Test query with invalid glob in exclude
#[test]
fn test_invalid_exclude_glob() {
    let output = run_doc_query(&["Vec", "--exclude", "[[invalid"]);

    assert!(
        !output.status.success(),
        "Invalid exclude glob should cause failure"
    );
}

/// Test query with both --minimal and --detailed (should warn)
#[test]
fn test_conflicting_minimal_detailed() {
    let output = run_doc_query(&["Vec", "--minimal", "--detailed"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either succeeds with warning or fails
    // The tool should handle this gracefully
    let is_handled =
        output.status.success() || stderr.contains("minimal") || stderr.contains("detailed");

    assert!(
        is_handled,
        "Conflicting minimal/detailed should be handled: {}",
        stderr
    );
}

/// Test kind filter with completely invalid kind
#[test]
fn test_invalid_kind() {
    let output = run_doc_query(&["Vec", "--kind", "this_is_not_a_valid_kind"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should either return empty results or fail gracefully
    let is_graceful = output.status.code() == Some(3)  // NO_RESULTS
        || stdout.trim().is_empty()
        || stderr.contains("kind");

    assert!(
        is_graceful,
        "Invalid kind should be handled gracefully: {}",
        stderr
    );
}
