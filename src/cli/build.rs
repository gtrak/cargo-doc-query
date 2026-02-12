use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use rustdoc_json::Builder;
use std::path::PathBuf;

use crate::cache::key::CacheKeyInputs;
use crate::cache::store::{CacheStore, SerializableCrateNode, SerializableIndex};
use crate::cli::Command;
use crate::index::graph::{CrateGraph, CrateNode};
use crate::parser::validate::validate_format_version;

pub struct BuildCommand {
    manifest_path: PathBuf,
    all_features: bool,
}

impl BuildCommand {
    pub fn new(manifest_path: PathBuf, all_features: bool) -> Self {
        Self {
            manifest_path,
            all_features,
        }
    }

    fn generate_rustdoc_json(
        &self,
        _deps: &[(String, String, Utf8PathBuf)], // Not used - use workspace manifest for all
    ) -> Result<Vec<(String, String, PathBuf)>> {
        let mut paths = Vec::new();

        println!("Generating rustdoc JSON for workspace packages...");

        // Use rustdoc-json with workspace manifest to document all workspace members
        // The package name is not needed when using workspace manifest - rustdoc handles it
        let builder = Builder::default()
            .toolchain("nightly")
            .manifest_path(&self.manifest_path);

        let builder = if self.all_features {
            builder.all_features(true)
        } else {
            builder
        };

        match builder.build() {
            Ok(path) => {
                println!("Successfully generated rustdoc JSON: {}", path.display());
                paths.push(("workspace".to_string(), "workspace".to_string(), path));
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to generate rustdoc JSON: {}", e));
            }
        }

        Ok(paths)
    }

    fn generate_serializable_index(
        &self,
        _deps: &[(String, String, Utf8PathBuf)],
        json_paths: &[(String, String, PathBuf)],
    ) -> SerializableIndex {
        // Convert graph to serializable format
        let mut nodes = Vec::new();

        // Add all nodes
        for (pkg_name, pkg_version, json_path) in json_paths {
            nodes.push(SerializableCrateNode {
                name: pkg_name.clone(),
                version: pkg_version.clone(),
                json_path: json_path.display().to_string(),
            });
        }

        SerializableIndex {
            format_version: 1,
            cache_key: String::new(), // Will be filled after cache key generation
            nodes,
            edges: Vec::new(), // Empty edges for now
        }
    }
}

impl Command for BuildCommand {
    fn execute(&self) -> Result<()> {
        println!("Discovering dependencies...");

        // 1. Generate cache key from project inputs (CACHE-01)
        let cache_inputs = CacheKeyInputs::from_project(&self.manifest_path)
            .context("Failed to create cache key")?;
        let cache_key = cache_inputs.generate_key();
        println!("Cache key: {}", &cache_key[..16]);

        // 2. Try to load from cache (CACHE-02)
        let cache_store = CacheStore::new().context("Failed to initialize cache store")?;

        if let Some(index) = cache_store.load(&cache_key)? {
            println!("Using cached index ({} crates)", index.nodes.len());
            return Ok(());
        }

        println!("No valid cache found, building index...");

        // 3. Generate rustdoc JSON for workspace (BUILD-02)
        let json_paths = self.generate_rustdoc_json(&[])?;

        println!("Generated rustdoc JSON");

        // 4. Parse and validate the JSON file (BUILD-05)
        let mut graph = CrateGraph::new();
        let json_paths_refs: Vec<_> = json_paths.iter().collect();
        for (pkg_name, pkg_version, json_path) in &json_paths_refs {
            println!("Processing {} v{}...", pkg_name, pkg_version);

            let json_str = std::fs::read_to_string(json_path)
                .with_context(|| format!("Failed to read {}", json_path.display()))?;

            // Format version validation - fail fast!
            validate_format_version(&json_str)
                .with_context(|| format!("Invalid format in {}", json_path.display()))?;

            // Add crate to graph
            let node = CrateNode {
                name: pkg_name.clone(),
                version: pkg_version.clone(),
                json_path: json_path.clone(),
            };
            graph.add_crate(node);
        }

        println!("Successfully indexed {} crates", graph.crate_count());

        // 5. Save to cache (CACHE-03)
        let mut serializable = self.generate_serializable_index(&[], &json_paths);
        serializable.cache_key = cache_key.clone();
        cache_store.save(&cache_key, &serializable)?;
        println!("Index cached successfully");

        println!("Build complete!");

        Ok(())
    }
}
