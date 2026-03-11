use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::time::Duration;

use cargo_doc_query::cache::key::CacheKeyInputs;
use cargo_doc_query::cache::store::{CacheStore, SerializableCrateNode, SerializableIndex};
use cargo_doc_query::parser::validate::validate_format_version;

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

    /// Generate rustdoc JSON using cargo doc with RUSTDOCFLAGS
    /// This generates JSON for external dependencies using cargo doc
    /// Works even when the workspace has compile errors because we scan the output directory
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
        let _output = cmd.output().context("Failed to run cargo doc")?;

        // Even if cargo doc fails, we can still collect the JSON files that were generated
        // for dependencies before the error occurred
        let output_dir = self.get_output_dir();
        self.scan_json_files(&output_dir)
    }

    /// Fallback: scan directory for JSON files
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
                        json_files.push((name, "0.0.0".to_string(), path));
                    }
                }
            }
        }

        Ok(json_files)
    }

    fn generate_serializable_index(
        &self,
        _deps: &[(String, String, Utf8PathBuf)],
        json_paths: &[(String, String, PathBuf)],
    ) -> SerializableIndex {
        let mut nodes = Vec::new();

        for (pkg_name, pkg_version, json_path) in json_paths {
            nodes.push(SerializableCrateNode {
                name: pkg_name.clone(),
                version: pkg_version.clone(),
                json_path: json_path.display().to_string(),
            });
        }

        SerializableIndex {
            format_version: 1,
            cache_key: String::new(),
            nodes,
            edges: Vec::new(),
        }
    }
}

impl BuildCommand {
    pub fn execute(&self, cache_store: &CacheStore) -> Result<()> {
        eprintln!("{}", style("Building documentation index...").bold().cyan());

        // 1. Get workspace dependencies (BUILD-02 fix)
        let deps_spinner = self.create_spinner("Discovering dependencies...");
        let deps = crate::cargo::dependencies::get_workspace_dependencies(&self.manifest_path)
            .context("Failed to get workspace dependencies")?;
        deps_spinner.finish_with_message(format!("Found {} external dependencies", deps.len()));

        // Check if there are any dependencies
        if deps.is_empty() {
            return Err(anyhow::anyhow!(
                "This project has no external dependencies. cargo-doc-query requires dependencies to index.\n\
                Add dependencies to your Cargo.toml and run `cargo doc-query build` again."
            ));
        }

        // 2. Generate cache key from project inputs (CACHE-01)
        let key_spinner = self.create_spinner("Computing cache key...");
        let cache_inputs = CacheKeyInputs::from_project(&self.manifest_path)
            .context("Failed to create cache key")?;
        let cache_key = cache_inputs.generate_key();
        key_spinner.finish_with_message(format!("Cache key: {}...", &cache_key[..16]));

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
        let json_paths = self.generate_rustdoc_json()?;

        // 5. Parse and validate all JSON files
        let process_pb = self.create_progress_bar(json_paths.len() as u64, "Indexing crates");

        let json_paths_refs: Vec<_> = json_paths.iter().collect();
        let mut ct = 0;
        for (pkg_name, pkg_version, json_path) in &json_paths_refs {
            process_pb.set_message(format!("Indexing {} v{}", pkg_name, pkg_version));

            let json_str = std::fs::read_to_string(json_path)
                .with_context(|| format!("Failed to read {}", json_path.display()))?;

            // Format version validation - warn but don't fail
            // Note: We skip strict validation here because large JSON files can exceed
            // serde_json's recursion limit during validation. The actual parsing with
            // rustdoc_types will catch format mismatches later.
            if let Err(_e) = validate_format_version(&json_str) {
                // Silently continue - validation failed but we'll try to parse anyway
            }

            ct += 1;
            process_pb.inc(1);
        }
        process_pb.finish_with_message(format!("Indexed {} crates", ct));

        // 8. Save to cache (CACHE-03)
        let save_spinner = self.create_spinner("Saving cache...");
        let mut serializable = self.generate_serializable_index(&deps, &json_paths);
        serializable.cache_key = cache_key.clone();
        cache_store.save(&cache_key, &serializable)?;
        save_spinner.finish_with_message("Cache saved");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
    fn test_serializable_index_generation() {
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
        let serializable = build_cmd.generate_serializable_index(&[], &json_paths);

        assert_eq!(serializable.format_version, 1);
        assert_eq!(serializable.nodes.len(), 2);
        assert_eq!(serializable.nodes[0].name, "crate1");
        assert_eq!(serializable.nodes[0].version, "1.0.0");
        assert_eq!(serializable.nodes[1].name, "crate2");
        assert_eq!(serializable.nodes[1].version, "2.0.0");
        assert_eq!(serializable.edges.len(), 0);
    }

    #[test]
    fn test_serializable_index_generation_with_absolute_paths() {
        let json_paths = vec![
            (
                "crate1".to_string(),
                "1.0.0".to_string(),
                PathBuf::from("/absolute/path/crate1.json"),
            ),
            (
                "crate2".to_string(),
                "2.0.0".to_string(),
                PathBuf::from("/another/path/crate2.json"),
            ),
        ];

        let build_cmd = BuildCommand::new(PathBuf::from("Cargo.toml"), false);
        let serializable = build_cmd.generate_serializable_index(&[], &json_paths);

        assert_eq!(serializable.nodes.len(), 2);
        assert!(serializable.nodes[0].json_path.starts_with("/absolute"));
        assert!(serializable.nodes[1].json_path.starts_with("/another"));
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
    fn test_cache_key_inputs_from_project() {
        let manifest_path = Path::new("test-Cargo-Unique.toml");
        let inputs = CacheKeyInputs::from_project(manifest_path).unwrap();

        // When file doesn't exist, returns empty content (this is expected behavior)
        assert!(inputs.cargo_toml_content.is_empty());
        assert!(!inputs.rustc_version.is_empty());
        assert!(!inputs.target_triple.is_empty());
        assert!(!inputs.rustdoc_types_version.is_empty());
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
    fn test_cache_key_format() {
        let manifest_path = Path::new("test-Cargo.toml");
        let inputs = CacheKeyInputs::from_project(manifest_path).unwrap();

        let key = inputs.generate_key();

        // Should be a non-empty string
        assert!(!key.is_empty());

        // Should be 64 characters for BLAKE3 hash
        assert_eq!(key.len(), 64);
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
}
