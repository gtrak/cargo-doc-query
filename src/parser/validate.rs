use anyhow::{Context, Result};
use rustdoc_types::FORMAT_VERSION;
use serde_json::Value;

/// Validates rustdoc JSON format version against expected FORMAT_VERSION.
/// Returns Ok(()) if valid, Err with clear message if not.
pub fn validate_format_version(json_str: &str) -> Result<()> {
    let value: Value = serde_json::from_str(json_str).context("Failed to parse JSON")?;

    let version = value
        .get("format_version")
        .and_then(|v| v.as_u64())
        .context("Missing format_version in rustdoc JSON")?;

    if version != FORMAT_VERSION as u64 {
        anyhow::bail!(
            "Format version mismatch: expected {}, got {}.\n\
             This usually means your rustdoc-types crate version doesn't match your Rust compiler.\n\
             Try: cargo update -p rustdoc-types",
            FORMAT_VERSION, version
        );
    }

    Ok(())
}
