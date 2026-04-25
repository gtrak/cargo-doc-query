use anyhow::{Context, Result};
use postcard::{from_bytes, to_stdvec};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(test)]
use tempfile::TempDir;

/// Serializable index for disk storage
#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableIndex {
    pub format_version: u32, // Should be 2 for new format (no cache_key field)
    pub nodes: Vec<SerializableCrateNode>,
    pub edges: Vec<(usize, usize, String)>, // (from, to, edge_type)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableCrateNode {
    pub name: String,
    pub version: String,
    pub env_hash: String, // Hash of environment used for global cache lookup
}

/// Cache storage for the documentation index
// @lat: [[two-tier-caching#Two-Tier Caching]]
pub struct CacheStore {
    cache_dir: PathBuf,
}

impl CacheStore {
    #[cfg(not(test))]
    pub fn new() -> Result<Self> {
        let cache_dir = PathBuf::from("target/doc-query");
        std::fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        Ok(Self { cache_dir })
    }

    #[cfg(test)]
    pub fn new_temp() -> Result<Self> {
        let temp_dir = TempDir::new().context("Failed to create temp directory")?;

        Ok(Self {
            cache_dir: temp_dir.keep(),
        })
    }

    #[cfg(test)]
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new().context("Failed to create temp directory")?;

        Ok(Self {
            cache_dir: temp_dir.keep(),
        })
    }

    /// Save index to cache at fixed path "index.idx"
    // @lat: [[two-tier-caching#Invariants]]
    pub fn save(&self, index: &SerializableIndex) -> Result<PathBuf> {
        let data = to_stdvec(index).context("Failed to serialize index")?;

        let path = self.cache_dir.join("index.idx");
        std::fs::write(&path, &data).context("Failed to write cache file")?;

        Ok(path)
    }

    /// Try to load index from cache at fixed path "index.idx"
    /// Returns None if cache doesn't exist
    // @lat: [[two-tier-caching#Invariants]]
    pub fn load(&self) -> Result<Option<SerializableIndex>> {
        let path = self.cache_dir.join("index.idx");

        if !path.exists() {
            return Ok(None);
        }

        let data = std::fs::read(&path).context("Failed to read cache file")?;

        // Try to deserialize - if it fails, cache is corrupt
        match from_bytes(&data) {
            Ok(index) => Ok(Some(index)),
            Err(e) => {
                eprintln!(
                    "⚠ Warning: Cache file appears corrupt ({}), will rebuild...",
                    e
                );
                let _ = std::fs::remove_file(&path);
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_store_new_creates_directory() {
        let cache_store = CacheStore::new_temp().unwrap();
        let cache_dir = cache_store.cache_dir;

        // Verify directory was created
        assert!(cache_dir.exists());
    }

    #[test]
    fn test_cache_save_and_load() {
        let cache_store = CacheStore::new_temp().unwrap();

        let test_index = SerializableIndex {
            format_version: 2,
            nodes: vec![],
            edges: vec![],
        };

        let saved_path = cache_store.save(&test_index).unwrap();
        assert!(saved_path.exists());

        let loaded = cache_store.load().unwrap().unwrap();
        assert_eq!(loaded.format_version, 2);
        assert_eq!(loaded.nodes.len(), 0);
        assert_eq!(loaded.edges.len(), 0);
    }

    #[test]
    fn test_cache_load_nonexistent() {
        let cache_store = CacheStore::new_temp().unwrap();

        let loaded = cache_store.load().unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_serializable_index_roundtrip() {
        let original = SerializableIndex {
            format_version: 2,
            nodes: vec![
                SerializableCrateNode {
                    name: "crate1".to_string(),
                    version: "1.0.0".to_string(),
                    env_hash: "abc123def456".to_string(),
                },
                SerializableCrateNode {
                    name: "crate2".to_string(),
                    version: "2.0.0".to_string(),
                    env_hash: "xyz789uvw012".to_string(),
                },
            ],
            edges: vec![(0, 1, "normal".to_string()), (1, 0, "dev".to_string())],
        };

        let data = to_stdvec(&original).unwrap();

        let loaded: SerializableIndex = from_bytes(&data).unwrap();

        assert_eq!(loaded.format_version, original.format_version);
        assert_eq!(loaded.nodes.len(), original.nodes.len());
        assert_eq!(loaded.edges.len(), original.edges.len());

        // Compare edge weights
        assert_eq!(loaded.edges.len(), original.edges.len());
        for (i, (from, to, edge_type)) in loaded.edges.iter().enumerate() {
            assert_eq!(*from, original.edges[i].0);
            assert_eq!(*to, original.edges[i].1);
            assert_eq!(edge_type, &original.edges[i].2);
        }

        // Compare env_hash fields
        assert_eq!(loaded.nodes[0].env_hash, "abc123def456");
        assert_eq!(loaded.nodes[1].env_hash, "xyz789uvw012");
    }

    #[test]
    fn test_serializable_crate_node_fields() {
        let node = SerializableCrateNode {
            name: "test-crate".to_string(),
            version: "1.0.0".to_string(),
            env_hash: "hash123".to_string(),
        };

        assert_eq!(node.name, "test-crate");
        assert_eq!(node.version, "1.0.0");
        assert_eq!(node.env_hash, "hash123");
    }

    #[test]
    fn test_serializable_index_empty() {
        let index = SerializableIndex {
            format_version: 2,
            nodes: vec![],
            edges: vec![],
        };

        let data = to_stdvec(&index).unwrap();
        let loaded: SerializableIndex = from_bytes(&data).unwrap();

        assert_eq!(loaded.nodes.len(), 0);
        assert_eq!(loaded.edges.len(), 0);
        assert_eq!(loaded.format_version, 2);
    }
}
