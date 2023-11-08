use cargo::core::registry::PackageRegistry;
use cargo::core::{Dependency, PackageId, Registry, SourceId, Workspace};
use cargo::sources::source::QueryKind;
use cargo::util::cache_lock::CacheLockMode;
use cargo::util::command_prelude::root_manifest;
use cargo::{ops, CargoResult, Config};

use super::view::{pretty_view, suggest_cargo_tree};

pub fn info(spec: &str, config: &Config) -> CargoResult<()> {
    let mut registry = PackageRegistry::new(config)?;
    // Make sure we get the lock before we download anything.
    let _lock = config.acquire_package_cache_lock(CacheLockMode::DownloadExclusive)?;
    registry.lock_patches();

    let mut package_id = None;
    // If we can find it in workspace, use it as a specific version.
    if let Ok(root) = root_manifest(None, config) {
        let ws = Workspace::new(&root, config)?;
        if let Some(resolve) = ops::load_pkg_lockfile(&ws)? {
            if let Ok(p) = resolve.query(spec) {
                package_id = Some(p)
            }
        }
    }

    query_and_pretty_view(spec, package_id, config, registry)
}

// Query the package registry and pretty print the result.
// If package_id is None, find the latest version.
fn query_and_pretty_view(
    spec: &str,
    package_id: Option<PackageId>,
    config: &Config,
    mut registry: PackageRegistry,
) -> CargoResult<()> {
    let (source_id, from_workspace) = match package_id {
        Some(package_id) => (package_id.source_id(), true),
        None => (SourceId::crates_io(config)?, false),
    };
    // Query without version requirement to get all index summaries.
    let dep = Dependency::parse(spec, None, source_id)?;
    let summaries = loop {
        // Exact to avoid returning all for path/git
        match registry.query_vec(&dep, QueryKind::Exact) {
            std::task::Poll::Ready(res) => {
                break res?;
            }
            std::task::Poll::Pending => registry.block_until_ready()?,
        }
    };

    let package_id = package_id.unwrap_or_else(|| {
        // Find the latest version.
        let summary = summaries
            .iter()
            .max_by_key(|s| s.package_id().version())
            .unwrap();

        summary.package_id()
    });

    let package = registry.get(&[package_id])?;
    let package = package.get_one(package_id)?;

    let mut shell = config.shell();
    let stdout = shell.out();

    // Suggest the cargo tree command if the package is from workspace.
    if from_workspace {
        suggest_cargo_tree(package_id, stdout)?;
    }

    pretty_view(package, &summaries, stdout)?;

    Ok(())
}
