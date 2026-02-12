use anyhow::{Context, Result};
use cargo_metadata::{CargoOpt, MetadataCommand};

/// Returns a list of (package_name, package_version) for all dependencies
/// that are NOT workspace members (external dependencies only)
pub fn get_workspace_dependencies(
    manifest_path: &std::path::Path,
) -> Result<Vec<(String, String)>> {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_path)
        .features(CargoOpt::AllFeatures)
        .exec()
        .context("Failed to load cargo metadata")?;

    let mut deps = Vec::new();

    for package in &metadata.packages {
        // Skip workspace members, only get external dependencies
        if !metadata.workspace_members.contains(&package.id) {
            deps.push((package.name.clone(), package.version.to_string()));
        }
    }

    // Remove duplicates (same crate, different versions)
    deps.sort();
    deps.dedup();

    Ok(deps)
}
