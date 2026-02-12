use anyhow::{Context, Result};
use postcard::{from_bytes, to_stdvec};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub fn new() -> Result<Self> {
        let cache_dir = PathBuf::from("target/doc-query");
        std::fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        Ok(Self { cache_dir })
    }

    /// Save index to cache
    pub fn save(&self, cache_key: &str, index: &SerializableIndex) -> Result<PathBuf> {
        let data = to_stdvec(index).context("Failed to serialize index")?;

        let path = self.cache_dir.join(format!("{}.idx", cache_key));
        std::fs::write(&path, &data).context("Failed to write cache file")?;

        Ok(path)
    }

    /// Try to load index from cache
    pub fn load(&self, cache_key: &str) -> Result<Option<SerializableIndex>> {
        let path = self.cache_dir.join(format!("{}.idx", cache_key));

        if !path.exists() {
            return Ok(None);
        }

        let data = std::fs::read(&path).context("Failed to read cache file")?;

        let index = from_bytes(&data).context("Failed to deserialize index")?;

        Ok(Some(index))
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
                    Some(current_latest) => {
                        if mtime > *current_latest {
                            latest_mtime = Some(mtime);
                            latest_path = Some(path);
                        }
                    }
                }
            }
        }

        match latest_path {
            Some(path) => {
                let data = std::fs::read(&path).context("Failed to read cache file")?;
                let index = from_bytes(&data).context("Failed to deserialize index")?;
                Ok(Some(index))
            }
            None => Ok(None),
        }
    }

    /// Clear stdlib JSON (for testing or forced rebuild)
    pub fn clear_stdlib(&self) -> Result<()> {
        let stdlib_dir = self.cache_dir.join("stdlib");

        if stdlib_dir.exists() {
            std::fs::remove_dir_all(&stdlib_dir).context("Failed to remove stdlib directory")?;
            println!("Cleared stdlib JSON");
        }

        Ok(())
    }
}
