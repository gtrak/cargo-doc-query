// Query command implementation

use anyhow::{Context, Result};
use clap::Parser;
use std::str::FromStr;
use std::time::Instant;

use crate::cache::key::CacheKeyInputs;
use crate::cache::store::CacheStore;
use crate::cli::build::BuildCommand;
use crate::cli::Command;
use crate::query::engine::{QueryEngine, QueryKind, QueryOptions};

#[derive(Parser, Debug)]
pub struct QueryCommand {
    /// The path to query (e.g., std::vec::Vec)
    path: String,

    /// Limit to specific crate
    #[arg(long)]
    crate_name: Option<String>,

    /// What to include in output
    #[arg(
        long,
        value_parser = clap::builder::PossibleValuesParser::new([
            "docs",
            "private",
            "trait_parameterization",
        ])
    )]
    include: Vec<String>,

    /// Which kind of query (methods, traits, types, all)
    #[arg(long, default_value = "all")]
    kind: QueryKindArg,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryKindArg {
    Methods,
    Traits,
    Types,
    All,
}

impl FromStr for QueryKindArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "methods" => Ok(QueryKindArg::Methods),
            "traits" => Ok(QueryKindArg::Traits),
            "types" => Ok(QueryKindArg::Types),
            "all" => Ok(QueryKindArg::All),
            _ => Err(format!(
                "Invalid query kind: {}. Expected: methods, traits, types, all",
                s
            )),
        }
    }
}

impl std::fmt::Display for QueryKindArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryKindArg::Methods => write!(f, "methods"),
            QueryKindArg::Traits => write!(f, "traits"),
            QueryKindArg::Types => write!(f, "types"),
            QueryKindArg::All => write!(f, "all"),
        }
    }
}

impl QueryCommand {
    pub fn new(
        path: String,
        crate_name: Option<String>,
        include: Vec<String>,
        kind: QueryKindArg,
    ) -> Self {
        Self {
            path,
            crate_name,
            include,
            kind,
        }
    }

    fn parse_options(&self) -> QueryOptions {
        let include_docs = self.include.contains(&"docs".to_string());
        let include_private = self.include.contains(&"private".to_string());
        let _include_trait_param = self.include.contains(&"trait_parameterization".to_string());

        let kind = match self.kind {
            QueryKindArg::Methods => QueryKind::Methods,
            QueryKindArg::Traits => QueryKind::Traits,
            QueryKindArg::Types => QueryKind::Types,
            QueryKindArg::All => QueryKind::All,
        };

        QueryOptions::new(kind)
            .with_docs(include_docs)
            .with_private(include_private)
    }
}

impl Command for QueryCommand {
    fn execute(&self) -> Result<()> {
        // Discover manifest path (default to current directory)
        let manifest_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("Cargo.toml");

        // Generate expected cache key from manifest files
        let cache_inputs = CacheKeyInputs::from_project(&manifest_path)
            .context("Failed to create cache key inputs")?;
        let expected_key = cache_inputs.generate_key();

        // Check if rebuild is needed
        let cache_store = CacheStore::new().context("Failed to initialize cache store")?;

        let index = if let Some(current_index) = cache_store.load_current()? {
            // Compare cache keys
            if current_index.cache_key != expected_key {
                println!("Manifest changed, rebuilding index...");
                let build_cmd = BuildCommand::new(
                    manifest_path.with_file_name("Cargo.toml"),
                    false, // Use default features
                );
                build_cmd.execute().context("Rebuild failed")?;
                // Reload index after rebuild
                cache_store
                    .load(&expected_key)
                    .context("Failed to load rebuilt index")?
                    .ok_or_else(|| anyhow::anyhow!("No cached index found after rebuild"))?
            } else {
                // Cache key matches, use current index
                current_index
            }
        } else {
            // No cache exists, need to build
            println!("No index found, building...");
            let build_cmd = BuildCommand::new(
                manifest_path.with_file_name("Cargo.toml"),
                false, // Use default features
            );
            build_cmd.execute().context("Build failed")?;
            cache_store
                .load(&expected_key)
                .context("Failed to load built index")?
                .ok_or_else(|| anyhow::anyhow!("No cached index found after build"))?
        };

        println!("Loaded index ({} crates)", index.nodes.len());

        // Time the query execution
        let start = Instant::now();

        // Create query engine
        let mut engine = QueryEngine::new(index);

        // Parse options
        let options = self.parse_options();

        // Execute query
        let response = engine
            .query(&self.path, &options, self.crate_name.as_deref())
            .context("Query failed")?;

        let duration = start.elapsed();
        eprintln!("Query completed in {}ms", duration.as_millis());

        // Output JSON
        let json_output = serde_json::to_string_pretty(&response)
            .context("Failed to serialize response as JSON")?;

        println!("{}", json_output);

        Ok(())
    }
}
