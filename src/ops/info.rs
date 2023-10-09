use cargo::core::registry::PackageRegistry;
use cargo::core::{PackageId, SourceId};
use cargo::{CargoResult, Config};

pub fn info(spec: &str, config: &Config) -> CargoResult<()> {
    let source_id = SourceId::crates_io(config)?;
    let mut registry = PackageRegistry::new(config)?;
    registry.add_sources([source_id])?;
    // Make sure we get the lock before we download anything.
    let _lock = config.acquire_package_cache_lock()?;

    let package_id = PackageId::new(spec, "1.0.0", source_id).unwrap();

    let package = registry.get(&[package_id])?;

    let package = package.get_one(package_id)?;

    println!("{:?}", package.manifest());

    Ok(())
}
