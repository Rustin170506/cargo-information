use std::collections::HashSet;

use cargo::core::registry::PackageRegistry;
use cargo::core::{Dependency, SourceId};
use cargo::sources::source::QueryKind;
use cargo::{CargoResult, Config};

pub fn info(spec: &str, config: &Config) -> CargoResult<()> {
    let source_id = SourceId::crates_io(config)?;
    let mut registry = PackageRegistry::new(config)?;
    registry.add_sources([source_id])?;
    // Make sure we get the lock before we download anything.
    let _lock = config.acquire_package_cache_lock()?;

    let mut source = source_id.load(&config, &HashSet::new())?;
    let dep = Dependency::parse(spec, None, source_id)?;

    let summaries = loop {
        // Exact to avoid returning all for path/git
        match source.query_vec(&dep, QueryKind::Exact) {
            std::task::Poll::Ready(res) => {
                break res?;
            }
            std::task::Poll::Pending => source.block_until_ready()?,
        }
    };

    // Get the latest version.
    let summary = summaries.into_iter().next().unwrap();

    let package_id = summary.package_id();

    let package = registry.get(&[package_id])?;

    let package = package.get_one(package_id)?;

    println!("{:?}", package.manifest());

    Ok(())
}
