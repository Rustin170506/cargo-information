use cargo::core::registry::PackageRegistry;
use cargo::core::PackageId;
use cargo::{CargoResult, Config};

pub fn info(spec: &str, config: &Config) -> CargoResult<()> {
    let mut registry = PackageRegistry::new(config)?;
    let source_id = config.crates_io_source_id()?;
    let package_id = PackageId::new(spec, "1.0.0", source_id).unwrap();

    registry.add_sources([source_id])?;

    let p = registry.get(&[package_id])?;

    let p = p.get_one(package_id)?;

    println!("{:?}", p.manifest());

    Ok(())
}
