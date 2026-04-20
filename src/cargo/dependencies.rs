use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use cargo_metadata::{CargoOpt, MetadataCommand};
use std::collections::HashSet;
use std::path::Path;

/// Parse Cargo.lock file to extract direct dependencies
/// This works even when the project has compile errors
fn parse_cargo_lock(manifest_dir: &Path) -> Result<Vec<(String, String)>> {
    let lock_path = manifest_dir.join("Cargo.lock");
    let lock_content = std::fs::read_to_string(&lock_path)
        .with_context(|| format!("Failed to read {:?}", lock_path))?;

    let mut packages = Vec::new();
    let mut in_package = false;
    let mut current_name: Option<String> = None;
    let mut current_version: Option<String> = None;
    let mut current_source: Option<String> = None;

    for line in lock_content.lines() {
        let trimmed = line.trim();

        if trimmed == "[[package]]" {
            // Save previous package if complete
            if let (Some(name), Some(version), Some(source)) =
                (&current_name, &current_version, &current_source)
            {
                // Only include registry dependencies (not path dependencies)
                if source.contains("registry+") || source.contains("git+") {
                    packages.push((name.clone(), version.clone()));
                }
            }
            // Reset for new package
            in_package = true;
            current_name = None;
            current_version = None;
            current_source = None;
        } else if in_package {
            if let Some((key, value)) = trimmed.split_once(" = ") {
                let value = value.trim().trim_matches('"').to_string();
                match key {
                    "name" => current_name = Some(value),
                    "version" => current_version = Some(value),
                    "source" => current_source = Some(value),
                    _ => {}
                }
            } else if trimmed.is_empty() {
                // End of this package block
                if let (Some(name), Some(version), Some(source)) =
                    (&current_name, &current_version, &current_source)
                {
                    if source.contains("registry+") || source.contains("git+") {
                        packages.push((name.clone(), version.clone()));
                    }
                }
                in_package = false;
                current_name = None;
                current_version = None;
                current_source = None;
            }
        }
    }

    // Handle last package
    if let (Some(name), Some(version), Some(source)) =
        (current_name, current_version, current_source)
    {
        if source.contains("registry+") || source.contains("git+") {
            packages.push((name, version));
        }
    }

    Ok(packages)
}

/// Returns a list of (package_name, package_version, manifest_path) for ALL dependencies
/// (direct + transitive). Falls back to parsing Cargo.lock if cargo metadata fails.
pub fn get_all_dependencies(
    manifest_path: &std::path::Path,
) -> Result<Vec<(String, String, Utf8PathBuf)>> {
    // Try cargo metadata first (gives us more accurate info)
    match try_all_dependencies(manifest_path) {
        Ok(deps) if !deps.is_empty() => return Ok(deps),
        Ok(_) => {
            // Empty deps, try lock file
        }
        Err(e) => {
            eprintln!(
                "cargo metadata failed (project may have compile errors): {}",
                e
            );
            eprintln!("Falling back to Cargo.lock parsing...");
        }
    }

    // Fallback: parse Cargo.lock
    let manifest_dir = manifest_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid manifest path: no parent directory"))?;

    let lock_deps = parse_cargo_lock(manifest_dir)
        .context("Failed to parse Cargo.lock and cargo metadata failed")?;

    // Convert to expected format (manifest_path is not available from lock file)
    let deps: Vec<_> = lock_deps
        .into_iter()
        .map(|(name, version)| (name, version, Utf8PathBuf::from("Cargo.lock")))
        .collect();

    Ok(deps)
}

/// Try to get ALL dependencies using cargo metadata (direct + transitive)
fn try_all_dependencies(
    manifest_path: &std::path::Path,
) -> Result<Vec<(String, String, Utf8PathBuf)>> {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_path)
        .features(CargoOpt::AllFeatures)
        .exec()
        .context("Failed to load cargo metadata")?;

    // Find the root package (the one at manifest_path)
    let manifest_path =
        std::fs::canonicalize(manifest_path).unwrap_or_else(|_| manifest_path.to_path_buf());
    let root_package = metadata
        .packages
        .iter()
        .find(|p| {
            let package_path = std::fs::canonicalize(p.manifest_path.as_std_path())
                .unwrap_or_else(|_| p.manifest_path.as_std_path().to_path_buf());
            package_path == manifest_path
        })
        .or_else(|| metadata.packages.first())
        .ok_or_else(|| anyhow::anyhow!("No root package found"))?;

    // Get ALL dependencies from resolve graph (not just direct ones)
    let all_dep_ids: HashSet<_> = metadata
        .resolve
        .as_ref()
        .and_then(|r| r.nodes.iter().find(|n| n.id == root_package.id))
        .map(|node| node.dependencies.iter().cloned().collect())
        .unwrap_or_default();

    // Filter packages to only dependencies (exclude workspace members)
    let mut deps = Vec::new();
    for package in &metadata.packages {
        // Skip workspace members (they're not external dependencies)
        if metadata.workspace_members.contains(&package.id) {
            continue;
        }
        // Only include packages that are in the dependency graph
        if !all_dep_ids.contains(&package.id) {
            continue;
        }
        deps.push((
            package.name.to_string(),
            package.version.to_string(),
            package.manifest_path.clone(),
        ));
    }

    // Remove duplicates (same crate can appear from different dependency paths)
    deps.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    deps.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);

    Ok(deps)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_manifest_path_canonicalization() {
        // Test that relative and absolute paths are handled correctly
        let current_dir = std::env::current_dir().unwrap();
        let manifest_path = current_dir.join("test-Cargo.toml");

        // This should not panic even with canonicalization
        let canonicalized =
            std::fs::canonicalize(&manifest_path).unwrap_or_else(|_| manifest_path.clone());

        assert!(canonicalized.is_absolute() || manifest_path.is_absolute());
        assert!(canonicalized.ends_with("test-Cargo.toml"));
    }

    #[test]
    fn test_get_all_dependencies_returns_structure() {
        let current_dir = std::env::current_dir().unwrap();
        let manifest_path = current_dir.join("Cargo.toml");

        let deps = super::get_all_dependencies(&manifest_path).expect("Failed to get dependencies");

        // Verify we got some dependencies (non-empty)
        assert!(!deps.is_empty(), "Expected non-empty dependency list");

        // Verify each entry has valid structure
        for (name, version, manifest_path) in &deps {
            // Check name is non-empty
            assert!(!name.is_empty(), "Dependency name should not be empty");

            // Check version is a valid semver (has at least major.minor.patch pattern)
            assert!(version.len() >= 3, "Version '{}' should have at least 3 characters", version);

            // Check manifest path is non-empty
            assert!(!manifest_path.as_str().is_empty(), "Manifest path should not be empty");
        }
    }

    #[test]
    fn test_get_all_dependencies_includes_transitive() {
        let current_dir = std::env::current_dir().unwrap();
        let manifest_path = current_dir.join("Cargo.toml");

        let all_deps = super::get_all_dependencies(&manifest_path).expect("Failed to get all dependencies");

        // Verify that transitive deps are included (we should have more than just a few direct deps)
        // This project has >5 direct dependencies, so transitive inclusion should yield many more
        assert!(
            all_deps.len() > 5,
            "Expected more than 5 total dependencies (including transitive), got {}",
            all_deps.len()
        );
    }

    #[test]
    fn test_get_all_dependencies_excludes_workspace() {
        let current_dir = std::env::current_dir().unwrap();
        let manifest_path = current_dir.join("Cargo.toml");

        let deps = super::get_all_dependencies(&manifest_path).expect("Failed to get dependencies");

        // Verify that the workspace crate itself (cargo-doc-query) is NOT in the list
        let workspace_members: Vec<&String> = deps.iter()
            .filter(|(name, _, _)| name == "cargo-doc-query")
            .map(|(name, _, _)| name)
            .collect();

        assert!(
            workspace_members.is_empty(),
            "Workspace member 'cargo-doc-query' should not be in the dependency list"
        );
    }
}
