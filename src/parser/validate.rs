use anyhow::{Context, Result};
use rustdoc_types::FORMAT_VERSION;
use serde_json::Value;

use crate::parser::serde_helper::deserialize_with_stack;

/// Validates rustdoc JSON format version against expected FORMAT_VERSION.
/// Returns Ok(()) if valid, Err with clear message if not.
pub fn validate_format_version(json_str: &str) -> Result<()> {
    let value: Value =
        deserialize_with_stack(json_str).context("Failed to parse JSON for validation")?;

    // Check if format_version exists (older rustdoc JSON format)
    if let Some(version) = value.get("format_version").and_then(|v| v.as_u64()) {
        if version != FORMAT_VERSION as u64 {
            anyhow::bail!(
                "Format version mismatch: expected {}, got {}.\n\
                 This usually means your rustdoc-types crate version doesn't match your Rust compiler.\n\
                 Try: cargo update -p rustdoc-types",
                FORMAT_VERSION, version
            );
        }
    }

    // Newer rustdoc JSON format doesn't have format_version field
    // We just validate that the JSON has the expected structure
    if !value.get("root").is_some() {
        anyhow::bail!(
            "Invalid rustdoc JSON format: missing 'root' field.\n\
             Expected rustdoc JSON output with either 'format_version' (legacy) or 'root' (current) field."
        );
    }

    Ok(())
}
