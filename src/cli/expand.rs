// Expand command for type hierarchy exploration

use anyhow::{Context, Result};
use clap::Parser;
use std::time::Instant;

use crate::cli::Command;
use crate::types::expand::TokenConfig;

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

    /// Maximum tokens in output (approximate, default: unlimited)
    #[arg(long)]
    tokens: Option<usize>,

    /// Output minimal representation (signatures only, no field details)
    #[arg(long)]
    minimal: bool,

    /// Output as JSON instead of human-readable text
    #[arg(long)]
    pub json: bool,

    /// Suppress progress indicators and timing info
    #[arg(skip)]
    pub quiet: bool,
}

impl ExpandCommand {
    pub fn new(path: String, depth: u32, crate_name: Option<String>) -> Self {
        Self {
            path,
            depth,
            crate_name,
            tokens: None,
            minimal: false,
            json: false,
            quiet: false,
        }
    }

    /// Create from parsed arguments
    pub fn from_args(
        path: String,
        depth: u32,
        crate_name: Option<String>,
        tokens: Option<usize>,
        minimal: bool,
    ) -> Self {
        Self {
            path,
            depth,
            crate_name,
            tokens,
            minimal,
            json: false,
            quiet: false,
        }
    }

    /// Set quiet mode
    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
    }
}

impl Command for ExpandCommand {
    fn execute(&self) -> Result<()> {
        // Validate token budget
        if let Some(tokens) = self.tokens {
            if tokens < 100 {
                return Err(anyhow::anyhow!(
                    "Token budget too small, minimum is 100 (got {})",
                    tokens
                ));
            }
        }

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
                if !self.quiet {
                    println!("Manifest changed, rebuilding index...");
                }
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
            if !self.quiet {
                println!("No index found, building...");
            }
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

        if !self.quiet {
            println!("Loaded index ({} crates)", index.nodes.len());
        }

        // Create token config
        let token_config = TokenConfig::new()
            .with_budget(self.tokens)
            .with_minimal(self.minimal);

        // Time the expansion execution
        let start = Instant::now();

        // Use expand_type_with_config for token budgeting
        let expansion = crate::query::expand::expand_type_with_config(
            &self.path,
            self.depth,
            self.crate_name.as_deref(),
            token_config,
        )
        .context(format!("Expansion failed for path: {}", self.path))?;

        let duration = start.elapsed();

        // Print warnings if budget exceeded
        if expansion.budget_exceeded && !self.quiet {
            eprintln!("⚠ Warning: Token budget exceeded. Some types were truncated.");
            if !expansion.truncated_paths.is_empty() {
                eprintln!("  Truncated: {:?}", expansion.truncated_paths);
            }
        }

        // Print token count and timing to stderr (unless quiet)
        if !self.quiet {
            eprintln!(
                "Expansion completed in {}ms ({} tokens)",
                duration.as_millis(),
                expansion.token_count
            );
        }

        // Output based on format preference
        if self.json {
            // Output JSON
            let json_output = serde_json::to_string_pretty(&expansion)
                .context("Failed to serialize expansion as JSON")?;
            println!("{}", json_output);
        } else {
            // Output human-readable text
            crate::format::text::format_expand_result(&expansion, &self.path);
        }

        Ok(())
    }
}
