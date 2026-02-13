use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use rustdoc_json::Builder;
use std::panic;
use std::path::{Path, PathBuf};
use std::time::Duration;

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

    /// Create a progress bar with a nice style
    fn create_progress_bar(&self, len: u64, msg: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    /// Create a spinner for indeterminate progress
    fn create_spinner(&self, msg: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    fn find_cached_json(&self, package_name: &str, package_version: &str) -> Option<PathBuf> {
        // Convert package name to file name format (e.g., "serde_json" -> "serde_json")
        let json_name = package_name.replace("-", "_");

        // Search in ~/.cargo/registry/src for pre-generated JSON
        let cargo_home = std::env::var("CARGO_HOME")
            .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.cargo", h)))
            .unwrap_or_else(|_| String::from("~/.cargo"));

        let registry_src = PathBuf::from(cargo_home).join("registry/src");

        if !registry_src.exists() {
            return None;
        }

        // Look for the crate directory matching the name and version
        // Pattern: registry/src/*/package-name-version/target/doc/package_name.json
        if let Ok(entries) = std::fs::read_dir(&registry_src) {
            for entry in entries.flatten() {
                if let Ok(crates) = std::fs::read_dir(entry.path()) {
                    for crate_entry in crates.flatten() {
                        let crate_path = crate_entry.path();
                        let crate_name = crate_path.file_name()?.to_str()?;

                        // Check if this directory matches our package
                        if crate_name.starts_with(&format!("{}-{}", package_name, package_version))
                            || crate_name.starts_with(&format!(
                                "{}-{}",
                                package_name.replace("-", "_"),
                                package_version
                            ))
                        {
                            let json_path = crate_path
                                .join("target/doc")
                                .join(format!("{}.json", json_name));
                            if json_path.exists() {
                                // Verify format version is compatible
                                if let Ok(content) = std::fs::read_to_string(&json_path) {
                                    if let Ok(format_version) =
                                        Self::extract_format_version(&content)
                                    {
                                        if format_version == 57 {
                                            // Current supported version
                                            return Some(json_path);
                                        } else {
                                            eprintln!("⚠ Cached JSON for {} has format version {}, expected 57 (will regenerate)",
                                                package_name, format_version);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract format version from JSON content without full parsing
    fn extract_format_version(json_content: &str) -> Result<u32> {
        // Quick extraction: find "format_version":N pattern
        if let Some(pos) = json_content.find("\"format_version\"") {
            let after_key = &json_content[pos + 16..]; // Skip "format_version"
            if let Some(colon_pos) = after_key.find(':') {
                let after_colon = &after_key[colon_pos + 1..];
                // Extract number (handle whitespace)
                let number_str: String = after_colon
                    .chars()
                    .skip_while(|c| c.is_whitespace())
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(version) = number_str.parse::<u32>() {
                    return Ok(version);
                }
            }
        }
        Err(anyhow::anyhow!("format_version not found in JSON"))
    }

    fn generate_rustdoc_json(
        &self,
        deps: &[(String, String, Utf8PathBuf)], // External dependencies with manifest paths
    ) -> Result<Vec<(String, String, PathBuf)>> {
        let mut paths = Vec::new();
        let mut need_generation = Vec::new();

        // Progress bar for checking cache
        let cache_pb = self.create_progress_bar(deps.len() as u64, "Checking cache");

        // First, try to find cached JSON files
        for (package_name, package_version, manifest_path) in deps {
            if let Some(cached_path) = self.find_cached_json(package_name, package_version) {
                paths.push((package_name.clone(), package_version.clone(), cached_path));
            } else {
                need_generation.push((
                    package_name.clone(),
                    package_version.clone(),
                    manifest_path.clone(),
                ));
            }
            cache_pb.inc(1);
        }
        cache_pb.finish_with_message(format!(
            "{} cached, {} need generation",
            paths.len(),
            need_generation.len()
        ));

        // Generate JSON for crates that don't have cached versions
        if !need_generation.is_empty() {
            let gen_pb = self.create_progress_bar(need_generation.len() as u64, "Generating docs");

            for (package_name, package_version, manifest_path) in need_generation {
                gen_pb.set_message(format!("Generating {} v{}", package_name, package_version));

                // Use rustdoc-json with package-local manifest to document external dependencies
                let builder = Builder::default()
                    .toolchain("nightly")
                    .manifest_path(&manifest_path);

                let builder = if self.all_features {
                    builder.all_features(true)
                } else {
                    builder
                };

                // Wrap build in catch_unwind for graceful error handling
                match panic::catch_unwind(|| builder.build()) {
                    Ok(Ok(path)) => {
                        paths.push((package_name, package_version, path));
                    }
                    Ok(Err(_e)) => {
                        // Silently continue on error
                    }
                    Err(_) => {
                        // Silently continue on panic
                    }
                }
                gen_pb.inc(1);
            }
            gen_pb.finish_with_message(format!("Generated {} docs", paths.len()));
        }

        if paths.is_empty() {
            return Err(anyhow::anyhow!(
                "Failed to generate rustdoc JSON for any package"
            ));
        }

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
        eprintln!("{}", style("Building documentation index...").bold().cyan());

        // 1. Get workspace dependencies (BUILD-02 fix)
        let deps_spinner = self.create_spinner("Discovering dependencies...");
        let deps = crate::cargo::dependencies::get_workspace_dependencies(&self.manifest_path)
            .context("Failed to get workspace dependencies")?;
        deps_spinner.finish_with_message(format!("Found {} external dependencies", deps.len()));

        // 2. Generate cache key from project inputs (CACHE-01)
        let key_spinner = self.create_spinner("Computing cache key...");
        let cache_inputs = CacheKeyInputs::from_project(&self.manifest_path)
            .context("Failed to create cache key")?;
        let cache_key = cache_inputs.generate_key();
        key_spinner.finish_with_message(format!("Cache key: {}...", &cache_key[..16]));

        // 3. Try to load from cache (CACHE-02)
        let cache_store = CacheStore::new().context("Failed to initialize cache store")?;

        if let Some(index) = cache_store.load(&cache_key)? {
            eprintln!(
                "{}",
                style(format!(
                    "✓ Using cached index ({} crates)",
                    index.nodes.len()
                ))
                .green()
            );
            return Ok(());
        }

        eprintln!("{}", style("Cache miss, building index...").yellow());

        // 4. Generate rustdoc JSON for external dependencies (BUILD-02 fix)
        let json_paths = self.generate_rustdoc_json(&deps)?;

        // 5. Parse and validate all JSON files
        let mut graph = CrateGraph::new();
        let process_pb = self.create_progress_bar(json_paths.len() as u64, "Indexing crates");

        let json_paths_refs: Vec<_> = json_paths.iter().collect();
        for (pkg_name, pkg_version, json_path) in &json_paths_refs {
            process_pb.set_message(format!("Indexing {} v{}", pkg_name, pkg_version));

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
            process_pb.inc(1);
        }
        process_pb.finish_with_message(format!("Indexed {} crates", graph.crate_count()));

        // 8. Save to cache (CACHE-03)
        let save_spinner = self.create_spinner("Saving cache...");
        let mut serializable = self.generate_serializable_index(&deps, &json_paths);
        serializable.cache_key = cache_key.clone();
        cache_store.save(&cache_key, &serializable)?;
        save_spinner.finish_with_message("Cache saved");

        Ok(())
    }
}
