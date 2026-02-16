//! Shared test utilities for cargo-doc-query integration tests

use std::process::{Command, Output};

/// Helper to run cargo doc-query command
pub fn run_doc_query(args: &[&str]) -> Output {
    Command::new("cargo")
        .args(["run", "--quiet", "--", "query"])
        .args(args)
        .output()
        .expect("Failed to execute cargo doc-query")
}
