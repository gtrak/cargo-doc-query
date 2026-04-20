// Shared crate loading logic for QueryEngine and TypeExpander

use std::collections::HashMap;
use std::fs;

use anyhow::{Context, Result};
use rustdoc_types::Crate;

use crate::cache::global::{CrateCacheKey, GlobalCacheStore};
use crate::cache::store::SerializableIndex;
use crate::parser::serde_helper::deserialize_with_stack;

/// Shared loader for rustdoc JSON crates.
/// Encapsulates the common logic for loading crates into a HashMap cache.
pub struct CrateLoader<'a> {
    /// Reference to the crates HashMap where loaded crates are stored
    crates: &'a mut HashMap<String, Crate>,
}

impl<'a> CrateLoader<'a> {
    /// Create a new loader with reference to crates HashMap
    pub fn new(crates: &'a mut HashMap<String, Crate>) -> Self {
        Self { crates }
    }

    /// Load a crate's rustdoc JSON into memory.
    /// Returns Ok(true) if loaded or already present, Ok(false) if not in global cache.
    pub fn load_crate(
        &mut self,
        index: &SerializableIndex,
        crate_name: &str,
        crate_version: &str,
    ) -> Result<bool> {
        // Check if already loaded
        let key = format!("{}::{}", crate_name, crate_version);
        if self.crates.contains_key(&key) {
            return Ok(true);
        }

        // Find the crate node (for error messages and validation)
        let _crate_node = index
            .nodes
            .iter()
            .find(|n| n.name == crate_name && n.version == crate_version)
            .ok_or_else(|| anyhow::anyhow!("Crate {} v{} not found in index", crate_name, crate_version))?;

        // Resolve JSON path via global cache
        let cache_key = CrateCacheKey::from_crate(crate_name, crate_version)?;
        let global_store = GlobalCacheStore::new()?;
        let json_path = match global_store.get(&cache_key) {
            Some(path) => path,
            None => return Ok(false), // Not cached, skip gracefully
        };

        // Load rustdoc JSON
        let json_str = fs::read_to_string(&json_path).with_context(|| {
            format!("Failed to read rustdoc JSON from {}", json_path.display())
        })?;

        let krate: Crate = deserialize_with_stack(&json_str).with_context(|| {
            format!("Failed to parse rustdoc JSON from {}", json_path.display())
        })?;

        self.crates.insert(key, krate);
        Ok(true)
    }
}
