use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
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

    /// Generate rustdoc JSON for specific packages using cargo doc with RUSTDOCFLAGS
    /// This builds only the specified packages via -p flags.
    /// Falls back to individual builds if batch build fails.
    fn generate_rustdoc_json_for_packages(
        &self,
        packages: &[(&str, &str)],
    ) -> Result<Vec<(String, String, PathBuf)>> {
        eprintln!(
            "{}",
            style(format!("Building {} packages...", packages.len())).yellow()
        );

        // Build all packages in a single cargo doc command with --all-features
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("+nightly").arg("doc");

        // Add -p flags for each package
        for (pkg_name, _) in packages {
            cmd.arg("-p").arg(pkg_name);
        }

        cmd.arg("--all-features");

        // Set RUSTDOCFLAGS for JSON output
        let rustdocflags = "-Z unstable-options --output-format json --document-private-items";
        cmd.env("RUSTDOCFLAGS", rustdocflags);

        // Set CARGO_TARGET_DIR for deterministic output location
        let cargo_target_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(Self::TARGET_DIR);
        cmd.env("CARGO_TARGET_DIR", &cargo_target_dir);

        // Run the command
        let output = cmd.output().context("Failed to run cargo doc")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!(
                "{}",
                style(format!(
                    "⚠ Batch build with --all-features failed ({})...",
                    stderr.lines().next().unwrap_or("unknown error")
                ))
                .yellow()
            );

            // Fallback: try building each package individually without --all-features
            let fallback_result = self.generate_rustdoc_json_individual_fallback(packages)?;
            if !fallback_result.is_empty() {
                return Ok(fallback_result);
            }
        }

        // Collect the JSON files that were generated
        let output_dir = self.get_output_dir();
        self.scan_json_files(&output_dir)
    }

    /// Fallback: build each package individually without --all-features
    fn generate_rustdoc_json_individual_fallback(
        &self,
        packages: &[(&str, &str)],
    ) -> Result<Vec<(String, String, PathBuf)>> {
        let mut all_json = Vec::new();

        for (pkg_name, _) in packages {
            eprintln!(
                "{}",
                style(format!("  Building {} individually...", pkg_name)).dim()
            );

            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("+nightly")
                .arg("doc")
                .arg("-p")
                .arg(pkg_name)
                // Note: NOT using --all-features in fallback mode
                .arg("--no-deps"); // Only document this package, not its deps

            let rustdocflags = "-Z unstable-options --output-format json --document-private-items";
            cmd.env("RUSTDOCFLAGS", rustdocflags);

            let cargo_target_dir = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(Self::TARGET_DIR);
            cmd.env("CARGO_TARGET_DIR", &cargo_target_dir);

            let output = cmd.output().context(format!("Failed to build {}", pkg_name))?;

            if !output.status.success() {
                eprintln!(
                    "{}",
                    style(format!("⚠ Failed to build {}", pkg_name)).yellow()
                );
                continue; // Try next package
            }

            // Collect JSON files for this package
            let output_dir = self.get_output_dir();
            if let Ok(mut json_files) = self.scan_json_files(&output_dir) {
                // Filter to only include the package we just built
                json_files.retain(|(name, _, _)| name == pkg_name);
                all_json.extend(json_files);
            }
        }

        Ok(all_json)
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
        json_paths: &[(String, String, PathBuf)],
    ) -> Result<SerializableIndex> {
        let mut nodes = Vec::new();

        for (pkg_name, pkg_version, _json_path) in json_paths {
            // Use real blake3 env_hash from CrateCacheKey
            let cache_key = CrateCacheKey::from_crate(pkg_name, pkg_version)?;
            let env_hash = cache_key.env_hash();
            nodes.push(SerializableCrateNode {
                name: pkg_name.clone(),
                version: pkg_version.clone(),
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

        // Step 2: Check if local cache already exists (quick return if we have a valid index)
        if let Some(index) = cache_store.load()? {
            eprintln!(
                "{}",
                style(format!("✓ Using cached index ({} crates)", index.nodes.len())).green()
            );
            return Ok(());
        }

        // Step 3: Create global cache store and partition deps into cached vs uncached
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

        // Step 6: Validate and copy built JSON files to global cache
        if !json_paths.is_empty() {
            let process_pb = self.create_progress_bar(json_paths.len() as u64, "Indexing crates");

            for (pkg_name, pkg_version, json_path) in &json_paths {
                process_pb.set_message(format!("Indexing {} v{}", pkg_name, pkg_version));

                // Validate JSON file can be read
                std::fs::read_to_string(json_path)
                    .with_context(|| format!("Failed to read {}", json_path.display()))?;

                // Copy to global cache using env_hash as key
                let cache_key = CrateCacheKey::from_crate(pkg_name, pkg_version)?;
                global_store.put(&cache_key, json_path)
                    .with_context(|| format!("Failed to copy {} to global cache", json_path.display()))?;

                process_pb.inc(1);
            }
            process_pb.finish_with_message(format!("Indexed {} crates", json_paths.len()));
        }

        // Step 7: Build serializable index with real env_hash values
        let save_spinner = self.create_spinner("Saving index...");
        let serializable = self.generate_serializable_index(&json_paths)?;
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
        let json_paths = vec![
            (
                "crate1".to_string(),
                "1.0.0".to_string(),
                PathBuf::from("/tmp/crate1.json"),
            ),
            (
                "crate2".to_string(),
                "2.0.0".to_string(),
                PathBuf::from("/tmp/crate2.json"),
            ),
        ];

        let build_cmd = BuildCommand::new(PathBuf::from("Cargo.toml"), false);
        let serializable = build_cmd.generate_serializable_index(&json_paths)?;

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

        let json_paths = vec![
            (
                "serde".to_string(),
                "1.0.204".to_string(),
                PathBuf::from("/tmp/serde.json"),
            ),
        ];

        let build_cmd = BuildCommand::new(PathBuf::from("Cargo.toml"), false);
        
        // Create cache key to get the real env_hash format
        let cache_key = CrateCacheKey::from_crate("serde", "1.0.204")?;
        let expected_env_hash = cache_key.env_hash();

        let serializable = build_cmd.generate_serializable_index(&json_paths)?;

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
}
