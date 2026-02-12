// Expand command for type hierarchy exploration

use anyhow::{Context, Result};
use clap::Parser;
use std::time::Instant;

use crate::cli::Command;

#[derive(Parser, Debug)]
pub struct ExpandCommand {
    /// The type path to expand (e.g., anyhow::Error)
    path: String,

    /// Maximum recursion depth (default: 3)
    #[arg(long, default_value = "3")]
    depth: u32,

    /// Limit to specific crate
    #[arg(long)]
    crate_name: Option<String>,
}

impl ExpandCommand {
    pub fn new(path: String, depth: u32, crate_name: Option<String>) -> Self {
        Self {
            path,
            depth,
            crate_name,
        }
    }
}

impl Command for ExpandCommand {
    fn execute(&self) -> Result<()> {
        // Discover manifest path (default to current directory)
        let manifest_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("Cargo.toml");

        // Generate expected cache key from manifest files
        let cache_inputs = crate::cache::key::CacheKeyInputs::from_project(&manifest_path)
            .context("Failed to create cache key inputs")?;
        let expected_key = cache_inputs.generate_key();

        // Check if rebuild is needed
        let cache_store =
            crate::cache::store::CacheStore::new().context("Failed to initialize cache store")?;

        let index = if let Some(current_index) = cache_store.load_current()? {
            // Compare cache keys
            if current_index.cache_key != expected_key {
                println!("Manifest changed, rebuilding index...");
                let build_cmd = crate::cli::build::BuildCommand::new(
                    manifest_path.with_file_name("Cargo.toml"),
                    false,
                );
                build_cmd.execute().context("Rebuild failed")?;
                cache_store
                    .load(&expected_key)
                    .context("Failed to load rebuilt index")?
                    .ok_or_else(|| anyhow::anyhow!("No cached index found after rebuild"))?
            } else {
                current_index
            }
        } else {
            // No cache exists, need to build
            println!("No index found, building...");
            let build_cmd = crate::cli::build::BuildCommand::new(
                manifest_path.with_file_name("Cargo.toml"),
                false,
            );
            build_cmd.execute().context("Build failed")?;
            cache_store
                .load(&expected_key)
                .context("Failed to load built index")?
                .ok_or_else(|| anyhow::anyhow!("No cached index found after build"))?
        };

        println!("Loaded index ({} crates)", index.nodes.len());

        // Time the expansion execution
        let start = Instant::now();

        // Use expand_type function from query/expand module
        let expansion =
            crate::query::expand::expand_type(&self.path, self.depth, self.crate_name.as_deref())
                .context("Expansion failed")?;

        let duration = start.elapsed();
        eprintln!("Expansion completed in {}ms", duration.as_millis());

        // Output JSON
        let json_output = serde_json::to_string_pretty(&expansion)
            .context("Failed to serialize expansion as JSON")?;

        println!("{}", json_output);

        Ok(())
    }
}
