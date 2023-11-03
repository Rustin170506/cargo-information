use cargo::core::registry::PackageRegistry;
use cargo::core::{Dependency, QueryKind, Registry, SourceId, Workspace};
use cargo::util::command_prelude::root_manifest;
use cargo::{ops, CargoResult, Config};

use super::view::pretty_view;

/// Check the information about a package.
pub fn info(spec: &str, config: &Config) -> CargoResult<()> {
    match root_manifest(None, config) {
        Ok(root) => {
            let ws = Workspace::new(&root, config);
            match ws {
                Ok(ws) => search_in_workspace(spec, &ws, config),
                Err(_) => {
                    // If we can't load the workspace, then we're not in a workspace
                    // and we should search crates.io.
                    search_out_of_workspace(spec, config)
                }
            }
        }
        Err(_) => return search_out_of_workspace(spec, config),
    }
}

fn search_in_workspace(spec: &str, ws: &Workspace<'_>, config: &Config) -> CargoResult<()> {
    let mut registry = PackageRegistry::new(config)?;
    // Make sure we get the lock before we download anything.
    let _lock = config.acquire_package_cache_lock()?;
    registry.lock_patches();
    let resolve = match ops::load_pkg_lockfile(ws)? {
        Some(resolve) => resolve,
        None => return Ok(()),
    };

    let package_id = resolve.query(spec)?;

    let package = registry.get(&[package_id])?;

    let package = package.get_one(package_id)?;

    let mut shell = config.shell();
    let stdout = shell.out();

    // TODO: fix this summary.
    pretty_view(package, &[], stdout)?;

    Ok(())
}

fn search_out_of_workspace(spec: &str, config: &Config) -> CargoResult<()> {
    let source_id = SourceId::crates_io_maybe_sparse_http(config)?;
    let mut registry = PackageRegistry::new(config)?;

    // Make sure we get the lock before we download anything.
    let _lock = config.acquire_package_cache_lock()?;
    registry.lock_patches();

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

    // Find the latest version.
    let summary = summaries
        .iter()
        .max_by_key(|s| s.package_id().version())
        .unwrap();

    let package_id = summary.package_id();

    let package = registry.get(&[package_id])?;

    let package = package.get_one(package_id)?;

    let mut shell = config.shell();
    let stdout = shell.out();

    pretty_view(package, &summaries, stdout)?;

    Ok(())
}
