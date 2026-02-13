//! Helper functions for serde deserialization with stack growth protection

use anyhow::{Context, Result};
use serde::Deserialize;

/// Deserialize JSON with automatic stack growth to handle deeply nested structures
pub fn deserialize_with_stack<T>(json: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let mut deserializer = serde_json::Deserializer::from_str(json);
    deserializer.disable_recursion_limit();
    let deserializer = serde_stacker::Deserializer::new(&mut deserializer);

    T::deserialize(deserializer).context("Failed to deserialize JSON (even with extended stack)")
}
