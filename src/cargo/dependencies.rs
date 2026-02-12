use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use cargo_metadata::{CargoOpt, MetadataCommand};
use std::collections::HashSet;

/// Returns a list of (package_name, package_version, manifest_path) for DIRECT dependencies only
pub fn get_workspace_dependencies(
    manifest_path: &std::path::Path,
) -> Result<Vec<(String, String, Utf8PathBuf)>> {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_path)
        .features(CargoOpt::AllFeatures)
        .exec()
        .context("Failed to load cargo metadata")?;

    // Find the root package (the one at manifest_path)
    let root_package = metadata
        .packages
        .iter()
        .find(|p| p.manifest_path.as_std_path() == manifest_path)
        .or_else(|| metadata.packages.first())
        .ok_else(|| anyhow::anyhow!("No root package found"))?;

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
            package.name.clone(),
            package.version.to_string(),
            package.manifest_path.clone(),
        ));
    }

    Ok(deps)
}
