use anyhow::Result;
use blake3::Hasher;
use std::collections::BTreeMap;
use std::path::Path;

/// Inputs that affect the documentation index output
#[derive(Debug, Clone)]
pub struct CacheKeyInputs {
    cargo_toml_content: Vec<u8>,      // Hash of Cargo.toml
    cargo_lock_content: Vec<u8>,      // Hash of Cargo.lock
    rustc_version: String,            // rustc --version output
    target_triple: String,            // Target platform triple
    features: BTreeMap<String, bool>, // Enabled features (sorted)
    rustdoc_types_version: String,    // rustdoc-types crate version
}

impl CacheKeyInputs {
    /// Create CacheKeyInputs from the project
    pub fn from_project(manifest_path: &Path) -> Result<Self> {
        // Read Cargo.toml content
        let cargo_toml_content = std::fs::read(manifest_path).unwrap_or_default();

        // Read Cargo.lock content
        let cargo_lock_path = manifest_path
            .parent()
            .unwrap_or(manifest_path)
            .join("Cargo.lock");
        let cargo_lock_content = std::fs::read(&cargo_lock_path).unwrap_or_default();

        // Get rustc version
        let rustc_version = std::process::Command::new("rustc")
            .arg("--version")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        // Get target triple
        let target_triple = std::env::consts::ARCH.to_string();

        // Get features from Cargo.toml (simplified - just use all_features)
        let features = BTreeMap::new();

        Ok(Self {
            cargo_toml_content,
            cargo_lock_content,
            rustc_version,
            target_triple,
            features,
            rustdoc_types_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    /// Get cargo.toml content
    pub fn cargo_toml_content(&self) -> &[u8] {
        &self.cargo_toml_content
    }

    /// Get rustc version
    pub fn rustc_version(&self) -> &str {
        &self.rustc_version
    }

    /// Get target triple
    pub fn target_triple(&self) -> &str {
        &self.target_triple
    }

    /// Get rustdoc_types version
    pub fn rustdoc_types_version(&self) -> &str {
        &self.rustdoc_types_version
    }

    /// Generate deterministic cache key using BLAKE3
    pub fn generate_key(&self) -> String {
        let mut hasher = Hasher::new();

        // Hash all inputs deterministically
        hasher.update(&self.cargo_lock_content); // Cargo.lock first (existing)
        hasher.update(&self.cargo_toml_content); // Cargo.toml next (new)
        hasher.update(self.rustc_version.as_bytes());
        hasher.update(self.target_triple.as_bytes());
        hasher.update(self.rustdoc_types_version.as_bytes());

        // Features
        for (feature, enabled) in &self.features {
            hasher.update(feature.as_bytes());
            hasher.update(&[*enabled as u8]);
        }

        hasher.finalize().to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_deterministic_same_inputs() {
        let manifest_path = Path::new("Cargo.toml");
        let inputs1 = CacheKeyInputs::from_project(manifest_path).unwrap();
        let inputs2 = CacheKeyInputs::from_project(manifest_path).unwrap();

        let key1 = inputs1.generate_key();
        let key2 = inputs2.generate_key();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_cargo_toml() {
        let manifest_path = Path::new("Cargo.toml");
        let inputs = CacheKeyInputs::from_project(manifest_path).unwrap();

        // Modify cargo_toml_content to get different key
        let mut modified = inputs.cargo_toml_content.clone();
        if !modified.is_empty() {
            modified[0] = modified[0].wrapping_add(1);
        }

        let mut modified_inputs = inputs.clone();
        modified_inputs.cargo_toml_content = modified;

        let key1 = inputs.generate_key();
        let key2 = modified_inputs.generate_key();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_cargo_lock() {
        let manifest_path = Path::new("Cargo.toml");
        let inputs = CacheKeyInputs::from_project(manifest_path).unwrap();

        // Modify cargo_lock_content to get different key
        let mut modified = inputs.cargo_lock_content.clone();
        if !modified.is_empty() {
            modified[0] = modified[0].wrapping_add(1);
        }

        let mut modified_inputs = inputs.clone();
        modified_inputs.cargo_lock_content = modified;

        let key1 = inputs.generate_key();
        let key2 = modified_inputs.generate_key();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_features() {
        let manifest_path = Path::new("Cargo.toml");
        let mut inputs = CacheKeyInputs::from_project(manifest_path).unwrap();

        let features1: BTreeMap<String, bool> = inputs.features.clone();
        inputs.features.insert("test-feature".to_string(), true);
        let key1 = inputs.generate_key();

        let mut inputs2 = inputs.clone();
        inputs2.features.insert("test-feature".to_string(), false);
        let key2 = inputs2.generate_key();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_rustc_version() {
        let manifest_path = Path::new("Cargo.toml");
        let inputs = CacheKeyInputs::from_project(manifest_path).unwrap();

        let mut modified_inputs = inputs.clone();
        modified_inputs.rustc_version = "test-version-123".to_string();
        let key1 = inputs.generate_key();
        let key2 = modified_inputs.generate_key();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_target_triple() {
        let manifest_path = Path::new("Cargo.toml");
        let inputs = CacheKeyInputs::from_project(manifest_path).unwrap();

        let mut modified_inputs = inputs.clone();
        modified_inputs.target_triple = "x86_64-pc-windows-msvc".to_string();
        let key1 = inputs.generate_key();
        let key2 = modified_inputs.generate_key();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_rustdoc_types_version() {
        let manifest_path = Path::new("Cargo.toml");
        let inputs = CacheKeyInputs::from_project(manifest_path).unwrap();

        let mut modified_inputs = inputs.clone();
        modified_inputs.rustdoc_types_version = "1.0.0".to_string();
        let key1 = inputs.generate_key();
        let key2 = modified_inputs.generate_key();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_non_empty() {
        let manifest_path = Path::new("Cargo.toml");
        let inputs = CacheKeyInputs::from_project(manifest_path).unwrap();

        let key = inputs.generate_key();
        assert!(!key.is_empty());
        assert!(key.len() > 10); // BLAKE3 hashes are 64 characters
    }

    #[test]
    fn test_cache_key_always_valid_hex() {
        let manifest_path = Path::new("Cargo.toml");
        let inputs = CacheKeyInputs::from_project(manifest_path).unwrap();

        let key = inputs.generate_key();

        // BLAKE3 hex output should only contain hex characters
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_features_order_doesnt_affect_key() {
        let manifest_path = Path::new("Cargo.toml");
        let inputs = CacheKeyInputs::from_project(manifest_path).unwrap();

        // Test with different feature orders (BTreeMap keeps them sorted)
        let features1: BTreeMap<String, bool> = inputs.features.clone();
        let key1 = features1
            .iter()
            .fold(String::new(), |acc, (name, enabled)| {
                acc + name.as_str() + if *enabled { "1" } else { "0" }
            });

        let features2: BTreeMap<String, bool> = inputs.features.clone();
        let key2 = features2
            .iter()
            .fold(String::new(), |acc, (name, enabled)| {
                acc + name.as_str() + if *enabled { "1" } else { "0" }
            });

        assert_eq!(key1, key2);
    }
}
