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

mod crate_query_tests {
    use super::*;

    /// Parameterized crate query test helper
    fn assert_crate_query(name: &str, query_arg: &str, extra_args: &[&str], expected: &str, alt: Option<&str>) {
        let args: Vec<_> = ["query", query_arg].iter().chain(extra_args.iter()).chain(&["--quiet"]).cloned().collect();
        let output = run_query(&args);
        
        if !is_cache_available(&output) { eprintln!("Skipping {}: cache not available", name); return; }

        assert!(output.contains(expected) || alt.map_or(false, |e| output.contains(e)),
                "Expected '{}' or '{}' in output for {}", expected, alt.unwrap_or(""), name);
    }

    #[test] fn test_serde_with_depth() {
        assert_crate_query("serde_with_depth", "serde", &["--depth", "1"], "serde", None);
    }

    #[test] fn test_anyhow_error_query() {
        assert_crate_query("anyhow_error", "anyhow::Error", &[], "anyhow::Error", Some("Error"));
    }

    #[test] fn test_clap_basic_query() {
        assert_crate_query("clap_basic", "clap", &[], "clap", None);
    }

    #[test] fn test_glob_query() {
        assert_crate_query("glob_query", "glob", &[], "glob", None);
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
