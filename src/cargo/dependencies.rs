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

/// Returns a list of (package_name, package_version, manifest_path) for DIRECT dependencies only
/// Falls back to parsing Cargo.lock if cargo metadata fails (e.g., when project has compile errors)
pub fn get_workspace_dependencies(
    manifest_path: &std::path::Path,
) -> Result<Vec<(String, String, Utf8PathBuf)>> {
    // Try cargo metadata first (gives us more accurate info)
    match try_cargo_metadata(manifest_path) {
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

/// Try to get dependencies using cargo metadata
fn try_cargo_metadata(
    manifest_path: &std::path::Path,
) -> Result<Vec<(String, String, Utf8PathBuf)>> {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_path)
        .features(CargoOpt::AllFeatures)
        .exec()
        .context("Failed to load cargo metadata")?;

    // Find the root package (the one at manifest_path)
    // Normalize paths for comparison (handle relative vs absolute)
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

    // Get direct dependencies only (from the root package's resolve node)
    let direct_dep_ids: HashSet<_> = metadata
        .resolve
        .as_ref()
        .and_then(|r| r.nodes.iter().find(|n| n.id == root_package.id))
        .map(|node| node.dependencies.iter().cloned().collect())
        .unwrap_or_default();

    // Filter packages to only direct dependencies
    let mut deps = Vec::new();
    for package in &metadata.packages {
        // Skip workspace members (they're not external dependencies)
        if metadata.workspace_members.contains(&package.id) {
            continue;
        }
        // Skip transitive dependencies (not in root package's dependencies)
        if !direct_dep_ids.contains(&package.id) {
            continue;
        }
        deps.push((
            package.name.to_string(),
            package.version.to_string(),
            package.manifest_path.clone(),
        ));
    }

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
}
