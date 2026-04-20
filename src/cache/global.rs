use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use anyhow::Result;

/// Cache key identifying a specific crate with its build environment.
#[derive(Debug, Clone, PartialEq)]
pub struct CrateCacheKey {
    pub name: String,
    pub version: String,
    pub rustc_version: String,
    pub target_triple: String,
    pub features_hash: String,
}

impl CrateCacheKey {
    /// Creates a cache key from crate name and version.
    /// Automatically captures rustc version and target triple from the system.
    pub fn from_crate(name: &str, version: &str) -> Result<Self> {
        let rustc_version = get_rustc_version()?;
        let target_triple = get_target_triple();
        
        Ok(Self {
            name: name.to_string(),
            version: version.to_string(),
            rustc_version,
            target_triple,
            features_hash: "default-features".to_string(),
        })
    }

    /// Returns a blake3 hash of the environment components as a 64-char hex string.
    pub fn env_hash(&self) -> String {
        let env_string = format!(
            "{}|{}|{}",
            self.rustc_version, self.target_triple, self.features_hash
        );
        blake3::hash(env_string.as_bytes()).to_hex().to_string()
    }

    /// Returns the JSON filename for this crate (with hyphens replaced by underscores).
    pub fn json_filename(&self) -> String {
        format!("{}.json", self.name.replace("-", "_"))
    }
}

/// Statistics about the cache contents.
#[derive(Debug, Default)]
pub struct CacheStats {
    pub entry_count: usize,
    pub total_size_bytes: u64,
}

/// Global cache store for crate documentation JSON files.
pub struct GlobalCacheStore {
    cache_dir: PathBuf,
}

impl GlobalCacheStore {
    /// Creates a new cache store using the platform's standard cache directory.
    pub fn new() -> Result<Self> {
        let base = dirs::cache_dir().ok_or_else(|| anyhow::anyhow!("Failed to get cache dir"))?;
        let cache_dir = base.join("cargo-doc-query").join("crates");
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// Creates a new cache store with a custom directory.
    /// This is useful for testing or when you need to use a non-standard cache location.
    pub fn new_with_dir(dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&dir)?;
        Ok(Self { cache_dir: dir })
    }

    /// Returns the path to a cached file if it exists.
    pub fn get(&self, key: &CrateCacheKey) -> Option<PathBuf> {
        let path = self.resolve(key);
        path.exists().then_some(path)
    }

    /// Stores a file in the cache and returns the destination path.
    pub fn put(&self, key: &CrateCacheKey, src_path: &Path) -> Result<PathBuf> {
        let dest = self.resolve(key);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src_path, &dest)?;
        Ok(dest)
    }

    /// Returns the expected path for a cache entry (whether it exists or not).
    pub fn resolve(&self, key: &CrateCacheKey) -> PathBuf {
        self.cache_dir
            .join(&key.name)
            .join(&key.version)
            .join(key.env_hash())
            .join(key.json_filename())
    }

    /// Removes all cache entries and returns statistics about what was removed.
    pub fn clean(&self) -> Result<CacheStats> {
        let stats = self.stats()?;
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
        }
        fs::create_dir_all(&self.cache_dir)?;
        Ok(stats)
    }

    /// Returns statistics about the current cache contents.
    pub fn stats(&self) -> Result<CacheStats> {
        let mut entry_count = 0usize;
        let mut total_size_bytes = 0u64;

        if !self.cache_dir.exists() {
            return Ok(CacheStats { entry_count, total_size_bytes });
        }

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let crate_dir = entry.path();
            
            // Iterate through versions (directories under crate name)
            if let Ok(version_dirs) = fs::read_dir(&crate_dir) {
                for version_entry in version_dirs {
                    if let Ok(version_entry) = version_entry {
                        let version_dir = version_entry.path();
                        
                        // Iterate through env hashes (directories under version)
                        if let Ok(hash_dirs) = fs::read_dir(&version_dir) {
                            for hash_entry in hash_dirs {
                                if let Ok(hash_entry) = hash_entry {
                                    let hash_dir = hash_entry.path();
                                    
                                    // Count JSON files in the hash directory
                                    if let Ok(files) = fs::read_dir(&hash_dir) {
                                        for file_entry in files {
                                            if let Ok(file_entry) = file_entry {
                                                let file_path = file_entry.path();
                                                if file_path.extension().and_then(|e| e.to_str()) == Some("json") {
                                                    entry_count += 1;
                                                    total_size_bytes += file_entry.metadata()?.len();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(CacheStats { entry_count, total_size_bytes })
    }
}

/// Captures rustc version by running `rustc --version`.
fn get_rustc_version() -> Result<String> {
    let output = Command::new("rustc").arg("--version").output()?;
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(Into::into)
}

/// Captures the target triple by running `rustc -vV` and parsing the "host:" line.
fn get_target_triple() -> String {
    let output = Command::new("rustc").arg("-vV").output();
    
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("host:") {
                return line.trim_start_matches("host:").trim().to_string();
            }
        }
    }
    
    // Fallback to system default architecture
    std::env::consts::ARCH.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_crate_cache_key_from_crate() -> Result<()> {
        let key = CrateCacheKey::from_crate("serde", "1.0.204")?;
        
        assert_eq!(key.name, "serde");
        assert_eq!(key.version, "1.0.204");
        assert!(key.rustc_version.contains("rustc"));
        assert!(!key.target_triple.is_empty());
        assert_eq!(key.features_hash, "default-features");
        
        Ok(())
    }

    #[test]
    fn test_env_hash_deterministic() -> Result<()> {
        let key1 = CrateCacheKey::from_crate("serde", "1.0.204")?;
        let key2 = CrateCacheKey::from_crate("serde", "1.0.204")?;
        
        assert_eq!(key1.env_hash(), key2.env_hash());
        assert_eq!(key1.env_hash().len(), 64); // blake3 hex output
        
        Ok(())
    }

    #[test]
    fn test_env_hash_different_rustc() {
        let key1 = CrateCacheKey {
            name: "serde".to_string(),
            version: "1.0.204".to_string(),
            rustc_version: "rustc 1.80.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            features_hash: "all-features".to_string(),
        };
        
        let key2 = CrateCacheKey {
            name: "serde".to_string(),
            version: "1.0.204".to_string(),
            rustc_version: "rustc 1.81.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            features_hash: "all-features".to_string(),
        };
        
        assert_ne!(key1.env_hash(), key2.env_hash());
    }

    #[test]
    fn test_json_filename_simple() -> Result<()> {
        let key = CrateCacheKey::from_crate("serde", "1.0.204")?;
        assert_eq!(key.json_filename(), "serde.json");
        Ok(())
    }

    #[test]
    fn test_json_filename_with_hyphens() -> Result<()> {
        let key = CrateCacheKey::from_crate("rustdoc-types", "0.25.0")?;
        assert_eq!(key.json_filename(), "rustdoc_types.json");
        Ok(())
    }

    #[test]
    fn test_global_store_new_creates_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_path = temp_dir.path().join("cache");
        
        assert!(!cache_path.exists());
        
        let store = GlobalCacheStore::new_with_dir(cache_path.clone())?;
        
        assert!(store.cache_dir.exists());
        assert!(cache_path.exists());
        
        Ok(())
    }

    #[test]
    fn test_global_store_put_and_get() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;
        
        let key = CrateCacheKey::from_crate("serde", "1.0.204")?;
        
        // Create a temporary source file
        let src_file = temp_dir.path().join("source.json");
        fs::write(&src_file, r#"{"test": "data"}"#)?;
        
        // Put into cache
        let dest_path = store.put(&key, &src_file)?;
        assert!(dest_path.exists());
        assert!(fs::read_to_string(&dest_path)?.contains("test"));
        
        // Get from cache
        let cached_path = store.get(&key);
        assert!(cached_path.is_some());
        assert_eq!(cached_path, Some(dest_path));
        
        Ok(())
    }

    #[test]
    fn test_global_store_get_nonexistent() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;
        
        let key = CrateCacheKey::from_crate("nonexistent", "1.0.0")?;
        assert!(store.get(&key).is_none());
        
        Ok(())
    }

    #[test]
    fn test_global_store_resolve_path() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;
        
        let key = CrateCacheKey {
            name: "serde".to_string(),
            version: "1.0.204".to_string(),
            rustc_version: "rustc 1.80.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            features_hash: "all-features".to_string(),
        };
        
        let path = store.resolve(&key);
        
        // The full path is cache_dir/serde/1.0.204/<envhash>/serde.json
        let expected_path = temp_dir.path()
            .join("serde")
            .join("1.0.204")
            .join(key.env_hash())
            .join("serde.json");
        
        assert_eq!(path, expected_path);
        
        Ok(())
    }

    #[test]
    fn test_global_store_clean() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;
        
        let key = CrateCacheKey::from_crate("serde", "1.0.204")?;
        let src_file = temp_dir.path().join("source.json");
        fs::write(&src_file, r#"{"test": "data"}"#)?;
        store.put(&key, &src_file)?;
        
        assert!(store.get(&key).is_some());
        
        let stats = store.clean()?;
        assert!(stats.entry_count > 0);
        assert!(stats.total_size_bytes > 0);
        
        assert!(store.get(&key).is_none());
        
        Ok(())
    }

    #[test]
    fn test_global_store_stats_empty() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;
        
        let stats = store.stats()?;
        assert_eq!(stats.entry_count, 0);
        assert_eq!(stats.total_size_bytes, 0);
        
        Ok(())
    }

    #[test]
    fn test_global_store_stats_with_entries() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;
        
        // Create two entries with different rustc versions (different env hashes)
        let key1 = CrateCacheKey {
            name: "serde".to_string(),
            version: "1.0.204".to_string(),
            rustc_version: "rustc 1.80.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            features_hash: "all-features".to_string(),
        };
        
        let key2 = CrateCacheKey {
            name: "serde".to_string(),
            version: "1.0.205".to_string(),
            rustc_version: "rustc 1.80.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            features_hash: "all-features".to_string(),
        };
        
        let src_file = temp_dir.path().join("source.json");
        fs::write(&src_file, r#"{"test": "data"}"#)?;
        
        store.put(&key1, &src_file)?;
        store.put(&key2, &src_file)?;
        
        let stats = store.stats()?;
        assert_eq!(stats.entry_count, 2);
        assert!(stats.total_size_bytes > 0);
        
        Ok(())
    }

    #[test]
    fn test_features_hash_is_default_features() -> Result<()> {
        let key = CrateCacheKey::from_crate("serde", "1.0.204")?;
        assert_eq!(key.features_hash, "default-features");
        Ok(())
    }

    #[test]
    fn test_env_hash_changes_with_features_hash() {
        let key_all = CrateCacheKey {
            name: "serde".to_string(),
            version: "1.0.204".to_string(),
            rustc_version: "rustc 1.80.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            features_hash: "all-features".to_string(),
        };
        let key_default = CrateCacheKey {
            name: "serde".to_string(),
            version: "1.0.204".to_string(),
            rustc_version: "rustc 1.80.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            features_hash: "default-features".to_string(),
        };
        assert_ne!(key_all.env_hash(), key_default.env_hash(), 
            "Different features_hash should produce different env_hash");
    }
}
