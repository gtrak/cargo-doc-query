use anyhow::{Context, Result};
use postcard::{from_bytes, to_stdvec};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(test)]
use tempfile::TempDir;

/// Serializable index for disk storage
#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableIndex {
    pub format_version: u32,
    pub cache_key: String,
    pub nodes: Vec<SerializableCrateNode>,
    pub edges: Vec<(usize, usize, String)>, // (from, to, edge_type)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableCrateNode {
    pub name: String,
    pub version: String,
    pub json_path: String,
}

/// Cache storage for the documentation index
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

    /// Save index to cache
    pub fn save(&self, cache_key: &str, index: &SerializableIndex) -> Result<PathBuf> {
        let data = to_stdvec(index).context("Failed to serialize index")?;

        let path = self.cache_dir.join(format!("{}.idx", cache_key));
        std::fs::write(&path, &data).context("Failed to write cache file")?;

        Ok(path)
    }

    /// Try to load index from cache
    /// Returns None if cache doesn't exist
    /// Returns CacheError if cache exists but is corrupt (should trigger rebuild)
    pub fn load(&self, cache_key: &str) -> Result<Option<SerializableIndex>> {
        let path = self.cache_dir.join(format!("{}.idx", cache_key));

        if !path.exists() {
            return Ok(None);
        }

        let data = std::fs::read(&path).context("Failed to read cache file")?;

        // Try to deserialize - if it fails, cache is corrupt
        match from_bytes(&data) {
            Ok(index) => Ok(Some(index)),
            Err(e) => {
                // Log the corrupt cache file and delete it
                eprintln!(
                    "⚠ Warning: Cache file appears corrupt ({}), will rebuild...",
                    e
                );
                let _ = std::fs::remove_file(&path);
                Ok(None)
            }
        }
    }

    /// Load the most recent index from cache (latest cache file)
    pub fn load_current(&self) -> Result<Option<SerializableIndex>> {
        let entries =
            std::fs::read_dir(&self.cache_dir).context("Failed to read cache directory")?;

        let mut latest_mtime: Option<std::time::SystemTime> = None;
        let mut latest_path: Option<PathBuf> = None;

        for entry in entries {
            let entry = entry.context("Failed to read cache entry")?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("idx") {
                let metadata = entry.metadata().context("Failed to read file metadata")?;

                let mtime = metadata
                    .modified()
                    .context("Failed to get modification time")?;

                match &latest_mtime {
                    None => {
                        latest_mtime = Some(mtime);
                        latest_path = Some(path);
                    }
                    Some(latest_ref) if mtime > *latest_ref => {
                        latest_mtime = Some(mtime);
                        latest_path = Some(path);
                    }
                    _ => {}
                }
            }
        }

        match latest_path {
            None => Ok(None),
            Some(latest_path) => {
                let data = std::fs::read(&latest_path).context("Failed to read cache file")?;
                // Try to deserialize - if it fails, cache is corrupt
                match from_bytes(&data) {
                    Ok(index) => Ok(Some(index)),
                    Err(e) => {
                        // Log the corrupt cache file and delete it
                        eprintln!(
                            "⚠ Warning: Cache file appears corrupt ({}), will rebuild...",
                            e
                        );
                        let _ = std::fs::remove_file(&latest_path);
                        Ok(None)
                    }
                }
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
        let cache_key = "test-key";

        let test_index = SerializableIndex {
            format_version: 1,
            cache_key: cache_key.to_string(),
            nodes: vec![],
            edges: vec![],
        };

        let saved_path = cache_store.save(cache_key, &test_index).unwrap();
        assert!(saved_path.exists());

        let loaded = cache_store.load(cache_key).unwrap().unwrap();
        assert_eq!(loaded.cache_key, cache_key);
        assert_eq!(loaded.format_version, 1);
        assert_eq!(loaded.nodes.len(), 0);
        assert_eq!(loaded.edges.len(), 0);
    }

    #[test]
    fn test_cache_load_nonexistent_key() {
        let cache_store = CacheStore::new_temp().unwrap();
        let nonexistent_key = "nonexistent-key-12345";

        let loaded = cache_store.load(nonexistent_key).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_cache_load_current_empty_directory() {
        let cache_store = CacheStore::new_temp().unwrap();

        // Verify we're in a temp directory by checking cache_dir exists
        assert!(cache_store.cache_dir.exists());

        let loaded = cache_store.load_current().unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_cache_load_current_multiple_files() {
        let cache_store = CacheStore::new_temp().unwrap();
        let key1 = "test-key-1";
        let key2 = "test-key-2";

        let test_index = SerializableIndex {
            format_version: 1,
            cache_key: key1.to_string(),
            nodes: vec![],
            edges: vec![],
        };

        let test_index2 = SerializableIndex {
            format_version: 1,
            cache_key: key2.to_string(),
            nodes: vec![],
            edges: vec![],
        };

        // Save key2 first to ensure it's older (different modification time)
        cache_store.save(key2, &test_index2).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Then save key1
        cache_store.save(key1, &test_index).unwrap();

        let loaded = cache_store.load_current().unwrap().unwrap();
        assert_eq!(loaded.cache_key, key1);
    }

    #[test]
    fn test_cache_save_overwrites_existing() {
        let cache_key = format!("overwrite-test-{}", std::process::id());

        let cache_store = CacheStore::new_temp().unwrap();

        let test_index1 = SerializableIndex {
            format_version: 1,
            cache_key: cache_key.clone(),
            nodes: vec![],
            edges: vec![],
        };

        let test_index2 = SerializableIndex {
            format_version: 2,
            cache_key: cache_key.clone(),
            nodes: vec![],
            edges: vec![],
        };

        cache_store.save(&cache_key, &test_index1).unwrap();

        let loaded_before = cache_store.load(&cache_key).unwrap().unwrap();
        assert_eq!(loaded_before.format_version, 1);

        // Clean up any existing file before saving
        let existing_path = cache_store.cache_dir.join(format!("{}.idx", cache_key));
        let _ = std::fs::remove_file(&existing_path);

        cache_store.save(&cache_key, &test_index2).unwrap();

        let loaded_after = cache_store.load(&cache_key).unwrap().unwrap();
        assert_eq!(loaded_after.format_version, 2);
    }

    #[test]
    fn test_serializable_index_roundtrip() {
        let original = SerializableIndex {
            format_version: 1,
            cache_key: "test-key".to_string(),
            nodes: vec![
                SerializableCrateNode {
                    name: "crate1".to_string(),
                    version: "1.0.0".to_string(),
                    json_path: "/path/to/crate1.json".to_string(),
                },
                SerializableCrateNode {
                    name: "crate2".to_string(),
                    version: "2.0.0".to_string(),
                    json_path: "/path/to/crate2.json".to_string(),
                },
            ],
            edges: vec![(0, 1, "normal".to_string()), (1, 0, "dev".to_string())],
        };

        let data = to_stdvec(&original).unwrap();

        let loaded: SerializableIndex = from_bytes(&data).unwrap();

        assert_eq!(loaded.format_version, original.format_version);
        assert_eq!(loaded.cache_key, original.cache_key);
        assert_eq!(loaded.nodes.len(), original.nodes.len());
        assert_eq!(loaded.edges.len(), original.edges.len());

        // Compare edge weights
        assert_eq!(loaded.edges.len(), original.edges.len());
        for (i, (from, to, edge_type)) in loaded.edges.iter().enumerate() {
            assert_eq!(*from, original.edges[i].0);
            assert_eq!(*to, original.edges[i].1);
            assert_eq!(edge_type, &original.edges[i].2);
        }
    }

    #[test]
    fn test_serializable_crate_node_fields() {
        let node = SerializableCrateNode {
            name: "test-crate".to_string(),
            version: "1.0.0".to_string(),
            json_path: "/path/to/crate.json".to_string(),
        };

        assert_eq!(node.name, "test-crate");
        assert_eq!(node.version, "1.0.0");
        assert_eq!(node.json_path, "/path/to/crate.json");
    }

    #[test]
    fn test_serializable_index_empty() {
        let index = SerializableIndex {
            format_version: 0,
            cache_key: String::new(),
            nodes: vec![],
            edges: vec![],
        };

        let data = to_stdvec(&index).unwrap();
        let loaded: SerializableIndex = from_bytes(&data).unwrap();

        assert_eq!(loaded.nodes.len(), 0);
        assert_eq!(loaded.edges.len(), 0);
        assert_eq!(loaded.format_version, 0);
    }
}
