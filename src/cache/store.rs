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
}
