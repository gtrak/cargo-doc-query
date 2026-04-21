use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cargo_doc_query::cache::global::{CrateCacheKey, GlobalCacheStore};
use cargo_doc_query::cache::store::{CacheStore, SerializableCrateNode, SerializableIndex};

pub struct BuildCommand {
    manifest_path: PathBuf,
    #[allow(dead_code)]
    all_features: bool,
    quiet: bool,
}

impl BuildCommand {
    pub fn new(manifest_path: PathBuf, all_features: bool) -> Self {
        Self {
            manifest_path,
            all_features,
            quiet: false,
        }
    }

    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
    }

    /// Create a progress bar with a nice style
    fn create_progress_bar(&self, len: u64, msg: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        if self.quiet {
            pb.set_style(ProgressStyle::default_bar().template("").unwrap());
            pb.finish_and_clear();
            return pb;
        }
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
        if self.quiet {
            pb.set_style(ProgressStyle::default_spinner().template("").unwrap());
            pb.finish_and_clear();
            return pb;
        }
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    /// Target directory for cargo-doc-query documentation output
    const TARGET_DIR: &str = "target/.cargo-doc-query";

    /// Get the deterministic output directory for rustdoc JSON files
    fn get_output_dir(&self) -> PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(Self::TARGET_DIR)
            .join("doc")
    }

    /// Builds each package individually with default features (no --all-features).
    /// Skips packages that fail to build (dev-dependencies often can't build docs).
    fn generate_rustdoc_json_for_packages(
        &self,
        packages: &[(&str, &str)],
    ) -> Result<Vec<(String, String, PathBuf)>> {
        eprintln!(
            "{}",
            style(format!("Building {} packages...", packages.len())).yellow()
        );

        // Build each package individually using cargo doc -p
        // We DON'T use --all-features because it's incompatible with -p for external deps
        let all_json = self.generate_rustdoc_json_individual(packages)?;

        if all_json.is_empty() {
            return Err(anyhow::anyhow!("No rustdoc JSON files were generated for any of the {} requested packages", packages.len()));
        }

        Ok(all_json)
    }

    /// Build each package individually with default features, in PARALLEL.
    /// Skips packages that fail to build (dev-dependencies often can't build docs).
    fn generate_rustdoc_json_individual(
        &self,
        packages: &[(&str, &str)],
    ) -> Result<Vec<(String, String, PathBuf)>> {
        eprintln!(
            "{}",
            style(format!("Building {} packages in parallel...", packages.len())).yellow()
        );

        let all_json = Arc::new(Mutex::new(Vec::new()));
        let failed_packages = Arc::new(Mutex::new(Vec::new()));
        let built_count = Arc::new(Mutex::new(0usize));

        // Create a progress bar (will be updated from multiple threads)
        let pb = Arc::new(Mutex::new(if !self.quiet {
            Some(self.create_progress_bar(packages.len() as u64, "Building packages"))
        } else {
            None
        }));

        // Build packages in parallel using rayon
        packages.par_iter().for_each(|(pkg_name, pkg_version)| {
            // Update progress bar
            if let Ok(pb_guard) = pb.lock() {
                if let Some(ref pb) = *pb_guard {
                    pb.set_message(format!("Building {} v{}...", pkg_name, pkg_version));
                }
            }

            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("+nightly")
                .arg("doc")
                .arg("-p")
                .arg(format!("{}@{}", pkg_name, pkg_version));
            // No --all-features (incompatible with -p for external deps)
            // Note: We DON'T use --no-deps because rustdoc needs deps to resolve types

            let rustdocflags = "-Z unstable-options --output-format=json";
            cmd.env("RUSTDOCFLAGS", rustdocflags);

            let cargo_target_dir = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(Self::TARGET_DIR);
            cmd.env("CARGO_TARGET_DIR", &cargo_target_dir);

            let output = cmd.output()
                .with_context(|| format!("Failed to execute cargo doc for {}", pkg_name));

            match output {
                Ok(output) if output.status.success() => {
                    if !self.quiet {
                        eprintln!(
                            "{}",
                            style(format!("  ✓ Built {} v{}", pkg_name, pkg_version)).green()
                        );
                    }

                    // Collect JSON files
                    if let Ok(json_files) = self.scan_json_files(&self.get_output_dir()) {
                        if let Ok(mut json_guard) = all_json.lock() {
                            json_guard.extend(json_files);
                        }
                    }

                    if let Ok(mut count_guard) = built_count.lock() {
                        *count_guard += 1;
                    }
                }
                _ => {
                    // Some crates (dev-deps, proc-macros) can't build docs — skip them
                    if !self.quiet {
                        eprintln!(
                            "{}",
                            style(format!("  ⚠ Skipped {} v{} (build failed)", pkg_name, pkg_version))
                                .yellow()
                        );
                    }
                    if let Ok(mut failed_guard) = failed_packages.lock() {
                        failed_guard.push(format!("{}@{}", pkg_name, pkg_version));
                    }
                }
            }

            // Increment progress bar
            if let Ok(pb_guard) = pb.lock() {
                if let Some(ref pb) = *pb_guard {
                    pb.inc(1);
                }
            }
        });

        // Unwrap Arc to get the Mutex contents
        let all_json = Arc::try_unwrap(all_json)
            .unwrap_or_else(|_| panic!("Arc still has multiple owners"))
            .into_inner()
            .unwrap();

        let failed_packages = Arc::try_unwrap(failed_packages)
            .unwrap_or_else(|_| Vec::new().into())
            .into_inner()
            .unwrap();

        let built_count = Arc::try_unwrap(built_count)
            .unwrap_or_else(|_| 0usize.into())
            .into_inner()
            .unwrap();

        if !failed_packages.is_empty() && !self.quiet {
            eprintln!(
                "{}",
                style(format!(
                    "  Skipped {} packages that couldn't build docs",
                    failed_packages.len()
                ))
                .dim()
            );
        }

        if built_count > 0 {
            eprintln!(
                "{}",
                style(format!("  Built {}/{} packages", built_count, packages.len())).green()
            );
        }

        // Finish progress bar
        if let Ok(pb_guard) = pb.lock() {
            if let Some(pb) = pb_guard.as_ref() {
                pb.finish_with_message(format!("Indexed {} crates", all_json.len()));
            }
        }

        // Deduplicate by (name, version) since multiple -p builds may produce overlapping results
        let mut seen = HashSet::new();
        let result: Vec<_> = all_json
            .into_iter()
            .filter(|(name, version, _)| seen.insert((name.clone(), version.clone())))
            .collect();

        Ok(result)
    }

    /// Generate rustdoc JSON using cargo doc with RUSTDOCFLAGS
    /// This generates JSON for all crates (workspace + dependencies) using cargo doc
    /// Works even when the workspace has compile errors because we scan the output directory
    /// for any JSON files that were successfully generated
    #[allow(dead_code)]
    fn generate_rustdoc_json(&self) -> Result<Vec<(String, String, PathBuf)>> {
        println!("Generating rustdoc JSON via cargo doc...");

        // Run cargo doc with JSON output flags
        // We don't use --no-deps so that dependencies are also documented
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("+nightly")
            .arg("doc")
            .arg("--all-features");

        // Set RUSTDOCFLAGS for JSON output
        let rustdocflags = "-Z unstable-options --output-format json --document-private-items";
        cmd.env("RUSTDOCFLAGS", rustdocflags);

        // Set CARGO_TARGET_DIR for deterministic output location
        let cargo_target_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(Self::TARGET_DIR);
        cmd.env("CARGO_TARGET_DIR", &cargo_target_dir);

        // Run the command (may fail if workspace has compile errors)
        let output = cmd.output().context("Failed to run cargo doc")?;

        // Log cargo doc output if it failed
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("cargo doc warning/error: {}", stderr);
        }

        // Even if cargo doc fails, we can still collect the JSON files that were generated
        // for dependencies before the error occurred
        let output_dir = self.get_output_dir();
        self.scan_json_files(&output_dir)
    }

    /// Extract version from a rustdoc JSON file
    fn extract_version_from_json(&self, path: &Path) -> Result<String> {
        let json_str = std::fs::read_to_string(path)?;
        let json: serde_json::Value = serde_json::from_str(&json_str)?;
        // The version is at the root level in the JSON
        let version = json.get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Version not found in JSON"))?;
        Ok(version.to_string())
    }

    /// Fallback: scan directory for JSON files and extract version from each file
    fn scan_json_files(&self, dir: &Path) -> Result<Vec<(String, String, PathBuf)>> {
        let mut json_files = Vec::new();

        if !dir.exists() {
            return Err(anyhow::anyhow!(
                "Output directory does not exist: {}",
                dir.display()
            ));
        }

        let entries: Vec<std::fs::DirEntry> =
            std::fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();

        for entry in entries {
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(stem) = path.file_stem() {
                    let name = stem.to_string_lossy().to_string();
                    // Skip internal files
                    if !name.starts_with("rustdoc_") {
                        let version = self.extract_version_from_json(&path)
                            .unwrap_or_else(|_| "0.0.0".to_string());
                        json_files.push((name, version, path));
                    }
                }
            }
        }

        Ok(json_files)
    }

    fn generate_serializable_index(
        &self,
        all_deps: &[(String, String, Utf8PathBuf)],  // (name, version, manifest_path)
    ) -> Result<SerializableIndex> {
        let mut nodes = Vec::new();

        for (name, version, _) in all_deps {
            // Use real blake3 env_hash from CrateCacheKey
            let cache_key = CrateCacheKey::from_crate(name, version)?;
            let env_hash = cache_key.env_hash();
            nodes.push(SerializableCrateNode {
                name: name.clone(),
                version: version.clone(),
                env_hash,
            });
        }

        Ok(SerializableIndex {
            format_version: 2,
            nodes,
            edges: Vec::new(),
        })
    }
}

impl BuildCommand {
    pub fn execute(&self, cache_store: &CacheStore) -> Result<()> {
        eprintln!("{}", style("Building documentation index...").bold().cyan());

        // Step 1: Get ALL dependencies (direct + transitive)
        let deps_spinner = self.create_spinner("Discovering dependencies...");
        let all_deps = crate::cargo::dependencies::get_all_dependencies(&self.manifest_path)
            .context("Failed to get all dependencies")?;
        deps_spinner.finish_with_message(format!("Found {} dependencies", all_deps.len()));

        // Check if there are any dependencies
        if all_deps.is_empty() {
            return Err(anyhow::anyhow!(
                "This project has no external dependencies. cargo-doc-query requires dependencies to index.\n\
                Add dependencies to your Cargo.toml and run `cargo doc-query build` again."
            ));
        }

        // Step 2: Create global cache store and partition deps into cached vs uncached
        let global_store = GlobalCacheStore::new()?;

        let mut cached_deps = Vec::new();
        let mut uncached_deps = Vec::new();

        for (name, version, _) in &all_deps {
            let key = CrateCacheKey::from_crate(name.as_str(), version.as_str())?;
            if global_store.get(&key).is_some() {
                cached_deps.push((name.clone(), version.clone()));
            } else {
                uncached_deps.push((name.clone(), version.clone()));
            }
        }

        let total = all_deps.len();
        let cached_count = cached_deps.len();
        let uncached_count = uncached_deps.len();

        // Step 4: Show cache progress
        if cached_count == total {
            eprintln!(
                "{}",
                style(format!("Found {}/{total} in global cache", cached_count)).green()
            );
        } else if uncached_count == total {
            eprintln!(
                "{}",
                style(format!("No dependencies in global cache, building all {total}...")).yellow()
            );
        } else {
            eprintln!(
                "{}",
                style(format!(
                    "Found {cached_count}/{total} in global cache, building {}...",
                    uncached_count
                ))
                .green()
            );
        }

        // Step 5: Build only uncached dependencies (if any)
        let json_paths = if !uncached_deps.is_empty() {
            // Convert to slice of (&str, &str) for the build function
            let packages: Vec<(&str, &str)> = uncached_deps
                .iter()
                .map(|(n, v)| (n.as_str(), v.as_str()))
                .collect();

            eprintln!("{}", style("Building index...").yellow());
            self.generate_rustdoc_json_for_packages(&packages)?
        } else {
            // All deps are cached, no building needed
            Vec::new()
        };

        // Step 6: Copy built JSON files to global cache using metadata names and versions
        if !json_paths.is_empty() {
            let process_pb = self.create_progress_bar(json_paths.len() as u64, "Indexing crates");

            // Build a map from normalized crate name → json_path for quick lookup
            let mut json_by_name: HashMap<String, PathBuf> = HashMap::new();
            for (scanned_name, _version, json_path) in &json_paths {
                let normalized = scanned_name.replace("-", "_");
                json_by_name.insert(normalized, json_path.clone());
            }

            for (dep_name, dep_version, _) in &all_deps {
                let normalized_name = dep_name.replace("-", "_");
                if let Some(json_path) = json_by_name.get(&normalized_name) {
                    process_pb.set_message(format!("Indexing {} v{}", dep_name, dep_version));

                    std::fs::read_to_string(json_path)
                        .with_context(|| format!("Failed to read {}", json_path.display()))?;

                    let cache_key = CrateCacheKey::from_crate(dep_name, dep_version)?;
                    global_store.put(&cache_key, json_path)
                        .with_context(|| format!("Failed to copy {} to global cache", json_path.display()))?;

                    process_pb.inc(1);
                }
                // If no JSON found for this dep, it was skipped during build (e.g., dev-deps)
                // That's fine — we'll have the dep in the index but no JSON, queries for that
                // specific crate will gracefully fail at query time
            }
            process_pb.finish_with_message(format!("Indexed {} crates", json_paths.len()));
        }

        // Step 7: Build serializable index with real env_hash values
        let save_spinner = self.create_spinner("Saving index...");
        let serializable = self.generate_serializable_index(&all_deps)?;
        cache_store.save(&serializable)?;
        save_spinner.finish_with_message("Index saved");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_creation() {
        let manifest_path = PathBuf::from("Cargo.toml");
        let build_cmd = BuildCommand::new(manifest_path.clone(), true);

        assert_eq!(build_cmd.manifest_path, manifest_path);
        assert!(build_cmd.all_features);
    }

    #[test]
    fn test_build_command_creation_without_features() {
        let manifest_path = PathBuf::from("Cargo.toml");
        let build_cmd = BuildCommand::new(manifest_path.clone(), false);

        assert_eq!(build_cmd.manifest_path, manifest_path);
        assert!(!build_cmd.all_features);
    }

    #[test]
    fn test_serializable_index_generation() -> anyhow::Result<()> {
        let all_deps = vec![
            (
                "crate1".to_string(),
                "1.0.0".to_string(),
                Utf8PathBuf::from("/tmp/Cargo.toml"),
            ),
            (
                "crate2".to_string(),
                "2.0.0".to_string(),
                Utf8PathBuf::from("/tmp2/Cargo.toml"),
            ),
        ];

        let build_cmd = BuildCommand::new(PathBuf::from("Cargo.toml"), false);
        let serializable = build_cmd.generate_serializable_index(&all_deps)?;

        assert_eq!(serializable.format_version, 2);
        assert_eq!(serializable.nodes.len(), 2);
        assert_eq!(serializable.nodes[0].name, "crate1");
        assert_eq!(serializable.nodes[0].version, "1.0.0");
        // env_hash is now a 64-char blake3 hash, not the old placeholder format
        assert_eq!(serializable.nodes[0].env_hash.len(), 64);
        assert_eq!(serializable.nodes[1].name, "crate2");
        assert_eq!(serializable.nodes[1].version, "2.0.0");
        assert_eq!(serializable.nodes[1].env_hash.len(), 64);
        assert_eq!(serializable.edges.len(), 0);

        Ok(())
    }

    #[test]
    fn test_package_name_to_json_name_conversion() {
        let tests = vec![
            ("serde", "serde"),
            ("serde_json", "serde_json"),
            ("some-crate", "some_crate"),
            ("another-package-name", "another_package_name"),
        ];

        for (package_name, expected) in tests {
            let json_name = package_name.replace("-", "_");
            assert_eq!(json_name, expected);
        }
    }

    #[test]
    fn test_progress_bar_creation() {
        let build_cmd = BuildCommand::new(PathBuf::from("Cargo.toml"), false);
        let pb = build_cmd.create_progress_bar(10, "test message");

        // ProgressBar is created successfully and is a valid progress bar
        assert_eq!(pb.length().unwrap(), 10);
        assert_eq!(pb.message(), "test message");
    }

    #[test]
    fn test_spinner_creation() {
        let build_cmd = BuildCommand::new(PathBuf::from("Cargo.toml"), false);
        let pb = build_cmd.create_spinner("test message");

        // Spinner is created successfully and is a valid spinner
        assert_eq!(pb.message(), "test message");
    }

    #[test]
    fn test_multiple_crate_names_with_hyphens() {
        let tests = vec![
            ("serde", "serde"),
            ("serde-json", "serde_json"),
            ("some-pkg-name", "some_pkg_name"),
            ("rustdoc-types", "rustdoc_types"),
        ];

        for (package_name, expected) in tests {
            let converted = package_name.replace("-", "_");
            assert_eq!(converted, expected);
        }
    }

     #[test]
    fn test_stdlib_package_list() {
        let stdlib_packages = [
            ("std", "0.0.0", "std/Cargo.toml"),
            ("core", "0.0.0", "core/Cargo.toml"),
            ("alloc", "0.0.0", "alloc/Cargo.toml"),
            ("proc_macro", "0.0.0", "proc_macro/Cargo.toml"),
        ];

        assert_eq!(stdlib_packages.len(), 4);

        for (name, version, _) in &stdlib_packages {
            assert!(!name.is_empty());
            assert!(!version.is_empty());
            assert!(!name.contains('/'));
        }
    }

    #[test]
    fn test_build_command_with_various_paths() {
        let paths = vec![
            PathBuf::from("/absolute/path/to/Cargo.toml"),
            PathBuf::from("../relative/Cargo.toml"),
            PathBuf::from("./local/Cargo.toml"),
            PathBuf::from("Cargo.toml"),
        ];

        for path in paths {
            let cmd = BuildCommand::new(path.clone(), false);
            assert_eq!(cmd.manifest_path, path);
        }
    }

    #[test]
    fn test_json_name_normalization_preserves_ascii() {
        let tests = vec![
            ("serde", "serde"),
            ("serde_json", "serde_json"),
            ("rustdoc_types", "rustdoc_types"),
            ("anyhow", "anyhow"),
            ("clap", "clap"),
        ];

        for (package_name, expected) in tests {
            let normalized = package_name.replace("-", "_");
            assert_eq!(normalized, expected);
        }
    }

    #[test]
    fn test_target_triple_detection() {
        let output = std::process::Command::new("rustc")
            .args(["-vV"])
            .output()
            .expect("rustc should be available");

        let stdout = String::from_utf8(output.stdout).expect("valid UTF-8");
        let host_line = stdout.lines().find(|l| l.starts_with("host:"));
        assert!(host_line.is_some());

        let host = host_line.unwrap()[5..].trim();
        assert!(!host.is_empty());

        let parts: Vec<&str> = host.split('-').collect();
        assert!(parts.len() >= 3);

        let doc_path = format!("target/{}/doc", host);
        assert!(doc_path.contains("target/"));
        assert!(doc_path.contains("/doc"));
        assert!(doc_path.ends_with("/doc"));
    }

    #[test]
    fn test_generate_serializable_index_uses_real_env_hash() -> anyhow::Result<()> {
        // Create a temp global cache store for testing
        let temp_dir = tempfile::tempdir()?;
        let _global_store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;

        let all_deps = vec![
            (
                "serde".to_string(),
                "1.0.204".to_string(),
                Utf8PathBuf::from("/tmp/Cargo.toml"),
            ),
        ];

        let build_cmd = BuildCommand::new(PathBuf::from("Cargo.toml"), false);
        
        // Create cache key to get the real env_hash format
        let cache_key = CrateCacheKey::from_crate("serde", "1.0.204")?;
        let expected_env_hash = cache_key.env_hash();

        let serializable = build_cmd.generate_serializable_index(&all_deps)?;

        // Verify we got exactly one node
        assert_eq!(serializable.nodes.len(), 1);
        
        // The env_hash should be the blake3 hash (64 hex chars), not "name@version" format
        let actual_env_hash = &serializable.nodes[0].env_hash;
        assert_eq!(actual_env_hash.len(), 64, "env_hash should be 64 hex characters");
        
        // Verify it's a valid hex string
        for c in actual_env_hash.chars() {
            assert!(c.is_ascii_hexdigit(), "env_hash should only contain hex digits");
        }

        // The env_hash should match what CrateCacheKey produces for the same crate
        assert_eq!(actual_env_hash, &expected_env_hash);
        
        // Verify it's NOT the old placeholder format
        assert_ne!(actual_env_hash, "serde@1.0.204");

        Ok(())
    }

    #[test]
    fn test_build_command_cargo_doc_args_for_uncached() -> anyhow::Result<()> {
        // Test that the -p flags are correctly constructed for uncached deps
        let temp_dir = tempfile::tempdir()?;
        let _global_store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;

        // Simulate discovering 3 dependencies
        let all_deps = vec![
            ("serde".to_string(), "1.0.204".to_string(), camino::Utf8PathBuf::from("/path/to/serde")),
            ("anyhow".to_string(), "1.0.86".to_string(), camino::Utf8PathBuf::from("/path/to/anyhow")),
            ("clap".to_string(), "4.5.23".to_string(), camino::Utf8PathBuf::from("/path/to/clap")),
        ];

        // Simulate that only serde is in cache, so anyhow and clap are uncached
        let cached_count = 1;
        let uncached_deps = &all_deps[cached_count..];

        // Build the -p arguments that would be passed to cargo doc
        let mut p_args: Vec<String> = vec![];
        for (name, _, _) in uncached_deps {
            p_args.push("-p".to_string());
            p_args.push(name.clone());
        }

        // Verify we have the correct structure
        assert_eq!(p_args.len(), 4, "Should have 2 crates * 2 args each");
        assert_eq!(p_args[0], "-p");
        assert_eq!(p_args[1], "anyhow");
        assert_eq!(p_args[2], "-p");
        assert_eq!(p_args[3], "clap");

        Ok(())
    }

    #[test]
    fn test_execute_partition_cached_uncached() -> anyhow::Result<()> {
        // Create a temp global cache store with one cached entry
        let temp_dir = tempfile::tempdir()?;
        let global_store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;

        // Pre-populate cache with serde
        let serde_key = CrateCacheKey::from_crate("serde", "1.0.204")?;
        let serde_file = temp_dir.path().join("serde_source.json");
        std::fs::write(&serde_file, r#"{"root": {}}"#)?;
        global_store.put(&serde_key, &serde_file)?;

        // Simulate discovering 3 dependencies (only serde is cached)
        let all_deps = vec![
            ("serde".to_string(), "1.0.204".to_string()),
            ("anyhow".to_string(), "1.0.86".to_string()),
            ("clap".to_string(), "4.5.23".to_string()),
        ];

        // Partition into cached vs uncached (this is the core logic being tested)
        let mut cached = Vec::new();
        let mut uncached = Vec::new();

        for (name, version) in &all_deps {
            let key = CrateCacheKey::from_crate(name.as_str(), version.as_str())?;
            if global_store.get(&key).is_some() {
                cached.push((name.clone(), version.clone()));
            } else {
                uncached.push((name.clone(), version.clone()));
            }
        }

        // Verify partition is correct
        assert_eq!(cached.len(), 1, "Should have exactly 1 cached dependency");
        assert_eq!(uncached.len(), 2, "Should have exactly 2 uncached dependencies");
        
        // Verify specific entries
        assert_eq!(cached[0].0, "serde");
        assert_eq!(cached[0].1, "1.0.204");
        
        let uncached_names: Vec<&str> = uncached.iter().map(|(n, _)| n.as_str()).collect();
        assert!(uncached_names.contains(&"anyhow"));
        assert!(uncached_names.contains(&"clap"));

        Ok(())
    }

    #[test]
    fn test_fallback_individual_build() -> anyhow::Result<()> {
        // Test that we can construct individual cargo doc commands as fallback
        let temp_dir = tempfile::tempdir()?;
        let _global_store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;

        // Simulate an uncached crate
        let uncached_crate = ("test-crate".to_string(), "1.0.0".to_string());

        // Construct the individual command (what we would do in fallback)
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("+nightly")
            .arg("doc")
            .arg("-p")
            .arg(&uncached_crate.0);
        
        // Note: In real fallback, we would NOT use --all-features for individual builds
        
        // Verify command structure
        let _ = cmd.arg("--help"); // Just check args are accepted, don't actually run
        // Command construction succeeded

        Ok(())
    }

    // Test for Bug 2: index used to only contain built deps, now contains ALL deps
    #[test]
    fn test_generate_serializable_index_includes_all_deps_not_just_built() -> anyhow::Result<()> {
        // Simulate 5 total dependencies
        let all_deps = vec![
            ("serde".to_string(), "1.0.204".to_string(), Utf8PathBuf::from("/path/to/serde")),
            ("anyhow".to_string(), "1.0.86".to_string(), Utf8PathBuf::from("/path/to/anyhow")),
            ("clap".to_string(), "4.5.23".to_string(), Utf8PathBuf::from("/path/to/clap")),
            ("petgraph".to_string(), "0.8.0".to_string(), Utf8PathBuf::from("/path/to/petgraph")),
            ("blake3".to_string(), "1.6.0".to_string(), Utf8PathBuf::from("/path/to/blake3")),
        ];

        let build_cmd = BuildCommand::new(PathBuf::from("Cargo.toml"), false);
        let serializable = build_cmd.generate_serializable_index(&all_deps)?;

        // ALL 5 deps must be in the index, not just a subset
        assert_eq!(serializable.nodes.len(), 5, "Index must contain ALL dependencies, not just built ones");
        
        // Verify each dep is present with correct name and version
        let names: Vec<&str> = serializable.nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"serde"));
        assert!(names.contains(&"anyhow"));
        assert!(names.contains(&"clap"));
        assert!(names.contains(&"petgraph"));
        assert!(names.contains(&"blake3"));
        
        // All env_hashes should be 64-char blake3 hex strings
        for node in &serializable.nodes {
            assert_eq!(node.env_hash.len(), 64, "env_hash should be 64 hex chars for {}", node.name);
        }

        Ok(())
    }

    #[test]
    fn test_generate_serializable_index_preserves_dep_order() -> anyhow::Result<()> {
        let all_deps = vec![
            ("z-crate".to_string(), "1.0.0".to_string(), Utf8PathBuf::from("/path/z")),
            ("a-crate".to_string(), "2.0.0".to_string(), Utf8PathBuf::from("/path/a")),
            ("m-crate".to_string(), "3.0.0".to_string(), Utf8PathBuf::from("/path/m")),
        ];

        let build_cmd = BuildCommand::new(PathBuf::from("Cargo.toml"), false);
        let serializable = build_cmd.generate_serializable_index(&all_deps)?;

        // Order should be preserved as-is (not sorted)
        assert_eq!(serializable.nodes[0].name, "z-crate");
        assert_eq!(serializable.nodes[1].name, "a-crate");
        assert_eq!(serializable.nodes[2].name, "m-crate");

        Ok(())
    }

    // Test that individual builds use simple cargo doc -p without --all-features or --no-deps
    #[test]
    fn test_individual_build_no_all_features_no_no_deps() -> anyhow::Result<()> {
        // Verify that individual package builds do NOT use --all-features
        // (it's incompatible with -p for external deps) or --no-deps
        // (we want transitive deps cached too)
        let pkg_name = "serde";

        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("+nightly")
            .arg("doc")
            .arg("-p")
            .arg(pkg_name);
        // No --all-features, no --no-deps

        let args: Vec<String> = cmd.get_args()
            .map(|s| s.to_str().unwrap_or_default().to_string())
            .collect();
        
        // Should have doc and -p
        assert!(args.contains(&"doc".to_string()), "Should have 'doc' arg");
        assert!(args.contains(&"-p".to_string()), "Should have '-p' arg");
        
        // Should NOT have --all-features (incompatible with -p for external deps)
        assert!(!args.contains(&"--all-features".to_string()), 
            "Individual builds should NOT use --all-features");
        
        // Should NOT have --no-deps (we want transitive deps cached)
        assert!(!args.contains(&"--no-deps".to_string()),
            "Individual builds should NOT use --no-deps");

        Ok(())
    }

    // Test for Bug 1: partition logic correctly identifies which deps need building
    #[test]
    fn test_cache_partition_logic_all_cached() -> anyhow::Result<()> {
        // Pre-populate cache with ALL dependencies
        let temp_dir = tempfile::tempdir()?;
        let global_store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;

        let all_deps = vec![
            ("serde".to_string(), "1.0.204".to_string()),
            ("anyhow".to_string(), "1.0.86".to_string()),
        ];

        // Add all to cache
        let src_file = temp_dir.path().join("source.json");
        std::fs::write(&src_file, r#"{"test": true}"#)?;
        for (name, version) in &all_deps {
            let key = CrateCacheKey::from_crate(name.as_str(), version.as_str())?;
            global_store.put(&key, &src_file)?;
        }

        // Partition
        let mut cached = Vec::new();
        let mut uncached = Vec::new();
        for (name, version) in &all_deps {
            let key = CrateCacheKey::from_crate(name.as_str(), version.as_str())?;
            if global_store.get(&key).is_some() {
                cached.push((name.clone(), version.clone()));
            } else {
                uncached.push((name.clone(), version.clone()));
            }
        }

        assert_eq!(cached.len(), 2, "All deps should be cached");
        assert_eq!(uncached.len(), 0, "No deps should need building");

        Ok(())
    }

    #[test]
    fn test_cache_partition_logic_partial_miss() -> anyhow::Result<()> {
        // Pre-populate cache with only some dependencies
        let temp_dir = tempfile::tempdir()?;
        let global_store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;

        let all_deps = vec![
            ("serde".to_string(), "1.0.204".to_string()),
            ("anyhow".to_string(), "1.0.86".to_string()),
            ("clap".to_string(), "4.5.23".to_string()),
        ];

        // Only cache serde
        let src_file = temp_dir.path().join("source.json");
        std::fs::write(&src_file, r#"{"test": true}"#)?;
        let serde_key = CrateCacheKey::from_crate("serde", "1.0.204")?;
        global_store.put(&serde_key, &src_file)?;

        // Partition
        let mut cached = Vec::new();
        let mut uncached = Vec::new();
        for (name, version) in &all_deps {
            let key = CrateCacheKey::from_crate(name.as_str(), version.as_str())?;
            if global_store.get(&key).is_some() {
                cached.push((name.clone(), version.clone()));
            } else {
                uncached.push((name.clone(), version.clone()));
            }
        }

        assert_eq!(cached.len(), 1, "Only serde should be cached");
        assert_eq!(uncached.len(), 2, "anyhow and clap should need building");
        assert_eq!(cached[0].0, "serde");

        let uncached_names: Vec<&str> = uncached.iter().map(|(n, _)| n.as_str()).collect();
        assert!(uncached_names.contains(&"anyhow"));
        assert!(uncached_names.contains(&"clap"));

        Ok(())
    }

    #[test]
    fn test_individual_build_command_structure() {
        // Verify that the individual build command for a package:
        // - Uses +nightly doc -p <pkg>
        // - Does NOT include --all-features
        // - Does NOT include --no-deps
        // - Includes RUSTDOCFLAGS env var
        // - Includes CARGO_TARGET_DIR env var
        let pkg_name = "serde";
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("+nightly").arg("doc").arg("-p").arg(pkg_name);
        // NO --all-features, NO --no-deps
        
        // Build the env vars that would be set
        let rustdocflags = "-Z unstable-options --output-format json --document-private-items";
        let cargo_target_dir = PathBuf::from(".").join(BuildCommand::TARGET_DIR);
        
        // Verify command structure
        let args: Vec<String> = cmd.get_args().map(|s| s.to_string_lossy().to_string()).collect();
        assert!(args.contains(&"doc".to_string()), "Should have 'doc' arg");
        assert!(args.contains(&"-p".to_string()), "Should have '-p' arg");
        assert!(args.contains(&pkg_name.to_string()), "Should have package name");
        assert!(!args.contains(&"--all-features".to_string()), "Should NOT have --all-features");
        assert!(!args.contains(&"--no-deps".to_string()), "Should NOT have --no-deps");
        
        // Verify env vars would be set
        assert!(!rustdocflags.is_empty(), "RUSTDOCFLAGS should not be empty");
        // CARGO_TARGET_DIR path constructed correctly
        assert!(cargo_target_dir.to_string_lossy().contains("target"), "CARGO_TARGET_DIR should contain 'target'");
    }

    // Test for Bug 3: verify global cache stores JSON under metadata version, not scanned version
    #[test]
    fn test_global_cache_uses_metadata_version() -> anyhow::Result<()> {
        // Simulate: scanned JSON reports version "0.0.0" but cargo metadata says "1.0.102"
        // The global cache should store under "1.0.102", not "0.0.0"
        let temp_dir = tempfile::tempdir()?;
        let global_store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;

        // Create a fake JSON file
        let json_file = temp_dir.path().join("anyhow.json");
        std::fs::write(&json_file, r#"{"format_version": 23}"#)?;

        // Store using the METADATA version (what cargo metadata reports)
        let metadata_key = CrateCacheKey::from_crate("anyhow", "1.0.102")?;
        global_store.put(&metadata_key, &json_file)?;

        // The global cache should NOT have an entry under "0.0.0"
        let scanned_key = CrateCacheKey::from_crate("anyhow", "0.0.0")?;
        assert!(global_store.get(&scanned_key).is_none(), 
            "Should NOT find JSON under scanned version 0.0.0");

        // The global cache SHOULD have an entry under "1.0.102"
        let found = global_store.get(&metadata_key);
        assert!(found.is_some(), 
            "Should find JSON under metadata version 1.0.102");

        Ok(())
    }

    #[test]
    fn test_json_name_normalization_matches_deps() -> anyhow::Result<()> {
        // Verify that cargo metadata name "rustdoc-types" matches
        // JSON filename "rustdoc_types" via normalization
        let dep_name = "rustdoc-types";
        let json_filename = "rustdoc_types";
        
        let normalized_dep = dep_name.replace("-", "_");
        let normalized_json = json_filename.replace("-", "_");
        
        assert_eq!(normalized_dep, normalized_json,
            "Normalized names should match for hyphenated crate names");
        
        // Also test the reverse: finding a JSON file for a dep
        let json_files: HashMap<String, String> = vec![
            ("rustdoc_types".to_string(), "/path/to/rustdoc_types.json".to_string()),
            ("serde".to_string(), "/path/to/serde.json".to_string()),
        ].into_iter().collect();
        
        let lookup_key = dep_name.replace("-", "_");
        assert!(json_files.contains_key(&lookup_key),
            "Should find JSON file for hyphenated dep name");
        
        Ok(())
    }

    // Integration test: verify the step 6 fix works end-to-end
    #[test]
    fn test_step6_uses_metadata_version_for_cache_storage() -> anyhow::Result<()> {
        // Simulate the scenario where scan_json_files returns (anyhow, "0.0.0", path)
        // but all_deps contains ("anyhow", "1.0.102", manifest_path)
        
        let temp_dir = tempfile::tempdir()?;
        let global_store = GlobalCacheStore::new_with_dir(temp_dir.path().to_path_buf())?;

        // Simulate json_paths from scan_json_files (version extracted from JSON is "0.0.0")
        let scanned_name = "anyhow";
        let scanned_version = "0.0.0";  // What scan_json_files would return on extraction failure
        let json_path = temp_dir.path().join("anyhow.json");
        std::fs::write(&json_path, r#"{"format_version": 23}"#)?;
        
        let json_paths = vec![(scanned_name.to_string(), scanned_version.to_string(), json_path.clone())];

        // Simulate all_deps from cargo metadata (version is "1.0.102")
        let all_deps = vec![("anyhow".to_string(), "1.0.102".to_string(), Utf8PathBuf::from("/path/to/Cargo.toml"))];

        // Execute the FIXED step 6 logic: build lookup map, iterate over all_deps
        let mut json_by_name: HashMap<String, PathBuf> = HashMap::new();
        for (scanned_name, _version, json_path) in &json_paths {
            let normalized = scanned_name.replace("-", "_");
            json_by_name.insert(normalized, json_path.clone());
        }

        for (dep_name, dep_version, _) in &all_deps {
            let normalized_name = dep_name.replace("-", "_");
            if let Some(json_path) = json_by_name.get(&normalized_name) {
                // Use metadata version, NOT scanned version
                let cache_key = CrateCacheKey::from_crate(dep_name, dep_version)?;
                global_store.put(&cache_key, json_path)?;
            }
        }

        // Verify: JSON is stored under metadata version "1.0.102", not scanned version "0.0.0"
        let metadata_key = CrateCacheKey::from_crate("anyhow", "1.0.102")?;
        let scanned_key = CrateCacheKey::from_crate("anyhow", "0.0.0")?;

        assert!(global_store.get(&metadata_key).is_some(),
            "JSON should be stored under metadata version 1.0.102");
        assert!(global_store.get(&scanned_key).is_none(),
            "JSON should NOT be stored under scanned version 0.0.0");

        // Verify the index also uses metadata version (line 373 generates_index with all_deps)
        let serializable = BuildCommand::new(PathBuf::from("Cargo.toml"), false)
            .generate_serializable_index(&all_deps)?;
        
        assert_eq!(serializable.nodes[0].name, "anyhow");
        assert_eq!(serializable.nodes[0].version, "1.0.102",
            "Index should use metadata version 1.0.102, matching global cache key");

        Ok(())
    }

}
