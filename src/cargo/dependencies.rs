use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use cargo_metadata::{CargoOpt, MetadataCommand};

/// Returns a list of (package_name, package_version, manifest_path) for all workspace members
pub fn get_workspace_dependencies(
    manifest_path: &std::path::Path,
) -> Result<Vec<(String, String, Utf8PathBuf)>> {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_path)
        .features(CargoOpt::AllFeatures)
        .exec()
        .context("Failed to load cargo metadata")?;

    let mut deps = Vec::new();

    // Skip workspace members, only get external dependencies
    // This avoids rustdoc-json errors when documenting registry crates from workspace root
    for package in &metadata.packages {
        if !metadata.workspace_members.contains(&package.id) {
            deps.push((
                package.name.clone(),
                package.version.to_string(),
                package.manifest_path.clone(),
            ));
        }
    }

    Ok(deps)
}
