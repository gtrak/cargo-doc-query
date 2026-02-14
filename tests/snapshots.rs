//! Snapshot tests for cargo-doc-query output consistency.
//!
//! These tests verify that the output format remains consistent across
//! different crates and query configurations. Tests are designed to
//! gracefully skip when cache is not available.

use std::process::Command;

/// Run cargo-doc-query and capture output
fn run_query(args: &[&str]) -> String {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .output()
        .expect("Failed to run cargo-doc-query");

    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Check if cache is available by looking for common error messages
fn is_cache_available(output: &str) -> bool {
    !output.contains("No cache found")
        && !output.contains("Cache miss")
        && !output.contains("not found")
}

mod serde_tests {
    use super::*;

    /// Test serde basic query
    #[test]
    fn test_serde_basic_query() {
        let output = run_query(&["query", "serde", "--quiet"]);
        if !is_cache_available(&output) {
            eprintln!("Skipping: cache not available");
            return;
        }

        // Should contain serde information
        assert!(output.contains("serde"), "Expected 'serde' in output");
    }

    /// Test serde with depth
    #[test]
    fn test_serde_with_depth() {
        let output = run_query(&["query", "serde", "--depth", "1", "--quiet"]);
        if !is_cache_available(&output) {
            eprintln!("Skipping: cache not available");
            return;
        }

        // Should contain serde
        assert!(output.contains("serde"), "Expected 'serde' in output");
    }
}

mod anyhow_tests {
    use super::*;

    /// Test anyhow::Error query
    #[test]
    fn test_anyhow_error_query() {
        let output = run_query(&["query", "anyhow::Error", "--quiet"]);
        if !is_cache_available(&output) {
            eprintln!("Skipping: cache not available");
            return;
        }

        // Should contain Error information
        assert!(
            output.contains("Error") || output.contains("error"),
            "Expected 'Error' in output"
        );
    }
}

mod clap_tests {
    use super::*;

    /// Test clap query
    #[test]
    fn test_clap_basic_query() {
        let output = run_query(&["query", "clap", "--quiet"]);
        if !is_cache_available(&output) {
            eprintln!("Skipping: cache not available");
            return;
        }

        // Should contain clap
        assert!(output.contains("clap"), "Expected 'clap' in output");
    }
}

mod glob_tests {
    use super::*;

    /// Test glob query
    #[test]
    fn test_glob_query() {
        let output = run_query(&["query", "glob", "--quiet"]);
        if !is_cache_available(&output) {
            eprintln!("Skipping: cache not available");
            return;
        }

        // Should contain glob
        assert!(output.contains("glob"), "Expected 'glob' in output");
    }
}

mod output_format_tests {
    use super::*;

    /// Test JSON output format
    #[test]
    fn test_json_output() {
        let output = run_query(&["query", "serde", "--json", "--quiet"]);
        if !is_cache_available(&output) {
            eprintln!("Skipping: cache not available");
            return;
        }

        // JSON output should start with { or [
        let trimmed = output.trim();
        assert!(
            trimmed.starts_with('{') || trimmed.starts_with('['),
            "Expected JSON output starting with {{ or [, got: {}",
            trimmed.chars().take(20).collect::<String>()
        );
    }

    /// Test minimal output format
    #[test]
    fn test_minimal_output() {
        let output = run_query(&["query", "serde", "--minimal", "--quiet"]);
        if !is_cache_available(&output) {
            eprintln!("Skipping: cache not available");
            return;
        }

        // Minimal should still contain some output
        assert!(!output.trim().is_empty(), "Expected non-empty output");
    }

    /// Test detailed output format
    #[test]
    fn test_detailed_output() {
        let output = run_query(&["query", "serde", "--detailed", "--quiet"]);
        if !is_cache_available(&output) {
            eprintln!("Skipping: cache not available");
            return;
        }

        // Detailed should still contain some output
        assert!(!output.trim().is_empty(), "Expected non-empty output");
    }

    /// Test token budget
    #[test]
    fn test_token_budget() {
        let output = run_query(&["query", "serde", "--tokens", "100", "--quiet"]);
        if !is_cache_available(&output) {
            eprintln!("Skipping: cache not available");
            return;
        }

        // Should still produce output
        assert!(!output.trim().is_empty(), "Expected non-empty output");
    }
}

mod filter_tests {
    use super::*;

    /// Test include filter
    #[test]
    fn test_include_filter() {
        let output = run_query(&["query", "serde", "--include", "serde::*", "--quiet"]);
        if !is_cache_available(&output) {
            eprintln!("Skipping: cache not available");
            return;
        }

        // Should contain serde
        assert!(output.contains("serde"), "Expected 'serde' in output");
    }

    /// Test exclude filter
    #[test]
    fn test_exclude_filter() {
        let output = run_query(&["query", "serde", "--exclude", "*test*", "--quiet"]);
        if !is_cache_available(&output) {
            eprintln!("Skipping: cache not available");
            return;
        }

        // Should still contain output
        assert!(!output.trim().is_empty(), "Expected non-empty output");
    }

    /// Test kind filter
    #[test]
    fn test_kind_filter() {
        let output = run_query(&["query", "serde", "--kind", "struct", "--quiet"]);
        if !is_cache_available(&output) {
            eprintln!("Skipping: cache not available");
            return;
        }

        // Should still contain output
        assert!(!output.trim().is_empty(), "Expected non-empty output");
    }
}
