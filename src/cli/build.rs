use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use rustdoc_json::Builder;
use std::panic;
use std::path::{Path, PathBuf};

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
        deps: &[(String, String, Utf8PathBuf)], // External dependencies with manifest paths
    ) -> Result<Vec<(String, String, PathBuf)>> {
        let mut paths = Vec::new();

        println!(
            "Generating rustdoc JSON for {} external dependency(s)...",
            deps.len()
        );

        for (package_name, package_version, manifest_path) in deps {
            println!(
                "Processing {} v{} at {}...",
                package_name, package_version, manifest_path
            );

            // Use rustdoc-json with package-local manifest to document external dependencies
            let builder = Builder::default()
                .toolchain("nightly")
                .manifest_path(manifest_path);

            let builder = if self.all_features {
                builder.all_features(true)
            } else {
                builder
            };

            // Wrap build in catch_unwind for graceful error handling
            // If a single crate fails, continue with others (graceful degradation)
            match panic::catch_unwind(|| builder.build()) {
                Ok(Ok(path)) => {
                    println!("✓ Successfully generated rustdoc JSON: {}", path.display());
                    paths.push((package_name.clone(), package_version.clone(), path));
                }
                Ok(Err(e)) => {
                    eprintln!(
                        "⚠ Failed to generate rustdoc JSON for {} v{}: {} (continuing with other crates)",
                        package_name, package_version, e
                    );
                }
                Err(_) => {
                    eprintln!(
                        "⚠ Rust panic while generating rustdoc JSON for {} v{} (continuing with other crates)",
                        package_name, package_version
                    );
                }
            }
        }

        if paths.is_empty() {
            return Err(anyhow::anyhow!(
                "Failed to generate rustdoc JSON for any package"
            ));
        }

        println!(
            "Successfully generated rustdoc JSON for {} crate(s)",
            paths.len()
        );

        Ok(paths)
    }

    /// Generate stdlib rustdoc JSON using rust-src component
    fn generate_stdlib_json(&self) -> Result<Vec<(String, String, PathBuf)>> {
        let stdlib_dir = PathBuf::from("target/doc-query/stdlib");

        // Create stdlib directory
        std::fs::create_dir_all(&stdlib_dir).context("Failed to create stdlib directory")?;

        println!("Generating stdlib rustdoc JSON...");

        // Find rust-src directory
        let rustup_home = std::env::var("RUSTUP_HOME")
            .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.rustup", h)))
            .unwrap_or_else(|_| String::from("~/.rustup"));

        let toolchain = "nightly"; // Try nightly first
        let rust_src_dir = PathBuf::from(&rustup_home)
            .join("toolchains")
            .join(format!("{}-x86_64-unknown-linux-gnu", toolchain))
            .join("lib/rustlib/src/rust/library");

        // Fallback to stable if nightly not available
        let rust_src_dir = if rust_src_dir.exists() {
            rust_src_dir
        } else {
            let stable_toolchain = "stable";
            PathBuf::from(&rustup_home)
                .join("toolchains")
                .join(format!("{}-x86_64-unknown-linux-gnu", stable_toolchain))
                .join("lib/rustlib/src/rust/library")
        };

        if !rust_src_dir.exists() {
            return Err(anyhow::anyhow!(
                "Rust source not found at {}. Run: rustup component add rust-src",
                rust_src_dir.display()
            ));
        }

        println!("Found rust-src at: {}", rust_src_dir.display());

        // Generate JSON for each stdlib crate
        let mut stdlib_crates = Vec::new();
        let stdlib_packages = [
            ("std", "0.0.0", "std/Cargo.toml"),
            ("core", "0.0.0", "core/Cargo.toml"),
            ("alloc", "0.0.0", "alloc/Cargo.toml"),
            ("proc_macro", "0.0.0", "proc_macro/Cargo.toml"),
        ];

        for (name, version, manifest_rel_path) in stdlib_packages {
            println!("Generating JSON for {}...", name);

            let manifest_path = rust_src_dir.join(manifest_rel_path);
            if !manifest_path.exists() {
                eprintln!(
                    "⚠ Manifest not found for {}: {} (skipping)",
                    name,
                    manifest_path.display()
                );
                continue;
            }

            let builder = Builder::default()
                .toolchain("nightly")
                .manifest_path(&manifest_path);

            // Wrap build in catch_unwind for graceful error handling
            match panic::catch_unwind(|| builder.build()) {
                Ok(Ok(path)) => {
                    // Copy JSON to our stdlib directory with proper naming
                    let dest_path = stdlib_dir.join(format!("{}.json", name));
                    match std::fs::copy(&path, &dest_path) {
                        Ok(_) => {
                            println!("✓ Generated stdlib JSON: {}", name);
                            stdlib_crates.push((name.to_string(), version.to_string(), dest_path));
                        }
                        Err(e) => {
                            eprintln!("⚠ Failed to copy JSON for {}: {}", name, e);
                        }
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("⚠ Failed to generate JSON for {}: {} (continuing)", name, e);
                }
                Err(_) => {
                    eprintln!("⚠ Panic while generating JSON for {} (continuing)", name);
                }
            }
        }

        if stdlib_crates.is_empty() {
            return Err(anyhow::anyhow!("No stdlib JSON files generated"));
        }

        println!("Generated stdlib JSON for {} crate(s)", stdlib_crates.len());

        Ok(stdlib_crates)
    }

    fn generate_serializable_index(
        &self,
        _deps: &[(String, String, Utf8PathBuf)],
        json_paths: &[(String, String, PathBuf)],
    ) -> SerializableIndex {
        // Convert graph to serializable format
        let mut nodes = Vec::new();

        // Add all nodes (use absolute paths)
        for (pkg_name, pkg_version, json_path) in json_paths {
            let absolute_path = if json_path.is_absolute() {
                json_path.clone()
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(json_path)
            };

            nodes.push(SerializableCrateNode {
                name: pkg_name.clone(),
                version: pkg_version.clone(),
                json_path: absolute_path.display().to_string(),
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

        // 1. Get workspace dependencies (BUILD-02 fix)
        let deps = crate::cargo::dependencies::get_workspace_dependencies(&self.manifest_path)
            .context("Failed to get workspace dependencies")?;
        println!("Found {} external dependencies", deps.len());

        // 2. Generate cache key from project inputs (CACHE-01)
        let cache_inputs = CacheKeyInputs::from_project(&self.manifest_path)
            .context("Failed to create cache key")?;
        let cache_key = cache_inputs.generate_key();
        println!("Cache key: {}", &cache_key[..16]);

        // 3. Try to load from cache (CACHE-02)
        let cache_store = CacheStore::new().context("Failed to initialize cache store")?;

        if let Some(index) = cache_store.load(&cache_key)? {
            println!("Using cached index ({} crates)", index.nodes.len());
            return Ok(());
        }

        println!("No valid cache found, building index...");

        // 4. Generate rustdoc JSON for stdlib
        let stdlib_json_paths = self
            .generate_stdlib_json()
            .context("Failed to generate stdlib JSON")?;

        // 5. Generate rustdoc JSON for external dependencies (BUILD-02 fix)
        let json_paths = self.generate_rustdoc_json(&deps)?;

        println!("Generated rustdoc JSON");

        // 6. Combine stdlib and external dependencies for index
        let all_json_paths: Vec<_> = stdlib_json_paths
            .into_iter()
            .chain(json_paths.into_iter())
            .collect();

        // 7. Parse and validate all JSON files
        let mut graph = CrateGraph::new();
        let json_paths_refs: Vec<_> = all_json_paths.iter().collect();
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

        // 8. Save to cache (CACHE-03)
        let mut serializable = self.generate_serializable_index(&deps, &all_json_paths);
        serializable.cache_key = cache_key.clone();
        cache_store.save(&cache_key, &serializable)?;
        println!("Index cached successfully");

        println!("Build complete!");

        Ok(())
    }
}
