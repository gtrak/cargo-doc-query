// Clean command for clearing the global documentation cache

use anyhow::Result;
use console::style;

use cargo_doc_query::cache::global::GlobalCacheStore;

pub struct CleanCommand {
    quiet: bool,
}

impl CleanCommand {
    pub fn new() -> Self {
        Self { quiet: false }
    }

    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
    }

    /// Execute the clean operation - clears the global per-crate cache
    pub fn execute(&self) -> Result<()> {
        let global_store = GlobalCacheStore::new()?;
        let stats = global_store.clean()?;
        
        if !self.quiet {
            if stats.entry_count > 0 {
                eprintln!(
                    "Cleared global cache: {} entries ({})",
                    style(stats.entry_count).bold(),
                    style(format_size(stats.total_size_bytes)).bold()
                );
            } else {
                eprintln!("{}", style("Global cache is already empty").dim());
            }
        }
        
        Ok(())
    }
}

/// Formats bytes into a human-readable string (e.g., "1.5 GB")
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_clean_command_new() {
        let cmd = CleanCommand::new();
        assert!(!cmd.quiet);
    }

    #[test]
    fn test_clean_command_set_quiet() {
        let mut cmd = CleanCommand::new();
        assert!(!cmd.quiet);
        cmd.set_quiet(true);
        assert!(cmd.quiet);
        cmd.set_quiet(false);
        assert!(!cmd.quiet);
    }

    #[test]
    fn test_clean_global_cache() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let global_store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;
        
        // Add a test file to the cache
        let test_file = temp_dir.path().join("test.json");
        fs::write(&test_file, r#"{"test": "data", "size": 1024}"#)?;
        
        let key = cargo_doc_query::cache::global::CrateCacheKey {
            name: "test-crate".to_string(),
            version: "1.0.0".to_string(),
            rustc_version: "rustc 1.80.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            features_hash: "all-features".to_string(),
        };
        
        global_store.put(&key, &test_file)?;
        
        // Verify cache has content before cleaning
        let stats_before = global_store.stats()?;
        assert!(stats_before.entry_count > 0);
        
        // Clean
        let clean_stats = global_store.clean()?;
        assert!(clean_stats.entry_count > 0);
        
        // Verify cache is empty after cleaning
        let final_stats = global_store.stats()?;
        assert_eq!(final_stats.entry_count, 0);
        
        Ok(())
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 bytes");
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_size((1536u64 * 1024 * 1024)), "1.5 GB");
    }
}
