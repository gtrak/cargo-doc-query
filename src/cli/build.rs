use anyhow::{Context, Result};
use rustdoc_json::Builder;
use std::path::PathBuf;

use crate::cargo::dependencies::get_workspace_dependencies;
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
        deps: &[(String, String)],
    ) -> Result<Vec<(String, String, PathBuf)>> {
        let mut paths = Vec::new();

        for (name, version) in deps {
            println!("Generating docs for {} v{}...", name, version);

            // Use rustdoc-json to generate documentation
            let builder = Builder::default()
                .toolchain("nightly")
                .manifest_path(&self.manifest_path)
                .package(&format!("{}@{}", name, version));

            let builder = if self.all_features {
                builder.all_features(true)
            } else {
                builder
            };

            match builder.build() {
                Ok(path) => {
                    paths.push((name.clone(), version.clone(), path));
                }
                Err(e) => {
                    eprintln!("Warning: Failed to document {}: {}", name, e);
                    // Continue with other crates - don't fail entire build
                }
            }
        }

        Ok(paths)
    }
}

impl Command for BuildCommand {
    fn execute(&self) -> Result<()> {
        println!("Discovering dependencies...");

        // 1. Discover dependencies (BUILD-02)
        let deps = get_workspace_dependencies(&self.manifest_path)
            .context("Failed to discover dependencies")?;

        println!("Found {} dependencies to document", deps.len());

        // 2. Generate rustdoc JSON for each dependency
        let json_paths = self.generate_rustdoc_json(&deps)?;

        println!("Generated rustdoc JSON for {} crates", json_paths.len());

        // 3. Parse and validate each JSON file (BUILD-05)
        let mut graph = CrateGraph::new();
        for (pkg_name, pkg_version, json_path) in json_paths {
            println!("Processing {} v{}...", pkg_name, pkg_version);

            let json_str = std::fs::read_to_string(&json_path)
                .with_context(|| format!("Failed to read {}", json_path.display()))?;

            // Format version validation - fail fast!
            validate_format_version(&json_str)
                .with_context(|| format!("Invalid format in {}", json_path.display()))?;

            // Add crate to graph
            let node = CrateNode {
                name: pkg_name.clone(),
                version: pkg_version.clone(),
                json_path,
            };
            graph.add_crate(node);
        }

        println!("Successfully indexed {} crates", graph.crate_count());
        println!("Build complete!");

        Ok(())
    }
}
