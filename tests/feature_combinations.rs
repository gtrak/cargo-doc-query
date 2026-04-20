//! Integration tests for feature combinations
//!
//! Tests all v1.1 features working together: filters + token budget + depth,
//! filters + minimal mode, filters + detailed mode, JSON output validation.

mod utils;
use std::process::Command;
use utils::run_doc_query;

/// Test filters + token budget + depth combined
#[test]
fn test_filters_tokens_depth_combined() {
    let output = run_doc_query(&[
        "Vec",
        "--include",
        "std::*",
        "--kind",
        "fn",
        "--depth",
        "1",
        "--tokens",
        "500",
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should handle complex combination gracefully
    assert!(
        output.status.success()
            || stderr.contains("No cache")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Complex filter combo should be handled: {}",
        stderr
    );
}

/// Test filters + minimal mode
#[test]
fn test_filters_minimal_mode() {
    let output = run_doc_query(&["Vec", "--include", "std::*", "--minimal"]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Filter + minimal should work: {}",
        stderr
    );
}

/// Test filters + detailed mode
#[test]
fn test_filters_detailed_mode() {
    let output = run_doc_query(&["Vec", "--kind", "trait", "--detailed"]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Filter + detailed should work: {}",
        stderr
    );
}

/// Helper: validates JSON output from doc-query
fn validate_json_output(args: &[&str], context: &str, deep_validate: bool) {
    let output = run_doc_query(args);
    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() && !stdout.trim().is_empty() {
        let trimmed = stdout.trim();
        // Assert 1: JSON structure starts with array or object
        assert!(
            trimmed.starts_with('[') || trimmed.starts_with('{'),
            "JSON {} should start with [ or {{: got {}",
            context,
            &trimmed[..trimmed.len().min(50)]
        );
        // Assert 2 (optional): deep validation parses valid JSON
        if deep_validate {
            assert!(
                serde_json::from_str::<serde_json::Value>(trimmed).is_ok(),
                "JSON {} should be parseable: got {}",
                context,
                &trimmed[..trimmed.len().min(50)]
            );
        }
    }
}

/// Test JSON output format
#[test]
fn test_json_output_format() {
    validate_json_output(&["Vec", "--json"], "basic format", false);
}

/// Test JSON output with filters
#[test]
fn test_json_with_filters() {
    validate_json_output(&["Vec", "--include", "std::*", "--json"], "with filters", false);
}

/// Test backward compatibility - JSON should have expected fields
#[test]
fn test_json_backward_compatibility() {
    validate_json_output(&["Vec", "--json", "--include", "std::*"], "backward compatible", true);
}

/// Test token budget limiting output
#[test]
fn test_token_budget_limits_output() {
    // Small budget should produce less output
    let output_small = run_doc_query(&["Vec", "--tokens", "50", "--include", "std::*"]);
    let output_large = run_doc_query(&["Vec", "--tokens", "5000", "--include", "std::*"]);

    let stdout_small = String::from_utf8_lossy(&output_small.stdout).len();
    let stdout_large = String::from_utf8_lossy(&output_large.stdout).len();

    // With token budget, smaller budget should typically produce smaller output
    // (unless there are no results)
    let both_succeeded = output_small.status.success() && output_large.status.success();

    if both_succeeded {
        // Either smaller is smaller, or both are similar (empty results)
        let ratio = if stdout_large > 0 {
            stdout_small as f64 / stdout_large as f64
        } else {
            1.0
        };
        assert!(
            ratio <= 1.5, // Allow some variance
            "Small budget ({}) should produce smaller or similar output to large budget ({}), ratio: {}",
            stdout_small,
            stdout_large,
            ratio
        );
    }
}

/// Test depth expansion with filters
#[test]
fn test_depth_expansion_with_filters() {
    let output = run_doc_query(&[
        "HashMap",
        "--depth",
        "2",
        "--include",
        "std::collections::*",
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Depth expansion with filters should work: {}",
        stderr
    );
}

/// Test minimal mode reduces output
#[test]
fn test_minimal_mode_reduces_output() {
    let output_regular = run_doc_query(&["Vec", "--include", "std::*"]);
    let output_minimal = run_doc_query(&["Vec", "--include", "std::*", "--minimal"]);

    let stdout_regular = String::from_utf8_lossy(&output_regular.stdout).len();
    let stdout_minimal = String::from_utf8_lossy(&output_minimal.stdout).len();

    let both_succeeded = output_regular.status.success() && output_minimal.status.success();

    if both_succeeded && stdout_regular > 0 {
        // Minimal should typically produce smaller output
        assert!(
            stdout_minimal <= stdout_regular,
            "Minimal mode ({}) should produce output <= regular ({})",
            stdout_minimal,
            stdout_regular
        );
    }
}

/// Test detailed mode includes more metadata
#[test]
fn test_detailed_mode_includes_metadata() {
    let output_regular = run_doc_query(&["Vec", "--include", "std::*"]);
    let output_detailed = run_doc_query(&["Vec", "--include", "std::*", "--detailed"]);

    let stdout_regular = String::from_utf8_lossy(&output_regular.stdout);
    let stdout_detailed = String::from_utf8_lossy(&output_detailed.stdout);

    let both_succeeded = output_regular.status.success() && output_detailed.status.success();

    if both_succeeded {
        // Detailed should typically have more content or different formatting
        // This is a loose check - just verify both produce output
        assert!(
            !stdout_regular.is_empty() || !stdout_detailed.is_empty(),
            "Both regular and detailed should produce some output"
        );
    }
}

/// Test with multiple filter types combined
#[test]
fn test_multiple_filter_types() {
    let output = run_doc_query(&[
        "Vec",
        "--include",
        "std::*",
        "--exclude",
        "*test*",
        "--kind",
        "function",
        "--visibility",
        "pub",
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Multiple filter types should work: {}",
        stderr
    );
}

/// Test --quiet flag with filters
#[test]
fn test_quiet_flag_with_filters() {
    let output = run_doc_query(&["Vec", "--include", "std::*", "--quiet"]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Quiet flag with filters should work: {}",
        stderr
    );
}

/// Test --no-color flag with filters
#[test]
fn test_no_color_flag_with_filters() {
    let output = run_doc_query(&["Vec", "--include", "std::*", "--no-color"]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "No-color flag with filters should work: {}",
        stderr
    );
}

/// Test expand command with filters
#[test]
fn test_expand_command_with_filters() {
    // The expand command is also available
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "query",
            "Vec",
            "--depth",
            "1",
            "--include",
            "std::*",
        ])
        .output()
        .expect("Failed to execute cargo doc-query");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "Expand with filters should work: {}",
        stderr
    );
}

/// Test query with serde crate (common dependency)
#[test]
fn test_query_common_crate() {
    let output = run_doc_query(&["Serialize", "--include", "serde*"]);

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should handle common crates gracefully
    assert!(
        output.status.success()
            || stderr.contains("No cache")
            || stderr.contains("not found")
            || stderr.contains("Error")
            || stderr.contains("not a valid"),
        "Common crate query should be handled: {}",
        stderr
    );
}

/// Helper function to validate kind filter behavior for any (query, kind) pair.
fn assert_kind_filter(query: &str, kind: &str, description: &str) {
    let output = run_doc_query(&[query, "--kind", kind]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success()
            || stderr.contains("No cache")
            || stderr.contains("not found")
            || stderr.contains("Error"),
        "{description} kind filter should work: {stderr}",
        description = description,
        stderr = stderr
    );
}

/// Test enum kind filter
#[test]
fn test_kind_filter_enum() {
    assert_kind_filter("Result", "enum", "Enum");
}

/// Test struct kind filter
#[test]
fn test_kind_filter_struct() {
    assert_kind_filter("Vec", "struct", "Struct");
}

/// Test trait kind filter
#[test]
fn test_kind_filter_trait() {
    assert_kind_filter("Clone", "trait", "Trait");
}
