use anyhow::Result;
use blake3::Hasher;
use std::collections::BTreeMap;
use std::path::Path;

/// Inputs that affect the documentation index output
#[derive(Debug)]
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
