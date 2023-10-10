use std::collections::HashSet;

use anstyle::{AnsiColor, Effects};
use cargo::core::registry::PackageRegistry;
use cargo::core::{Dependency, Package, SourceId};
use cargo::sources::source::QueryKind;
use cargo::util::style::{GOOD, NOP, WARN};
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

    // Find the latest version.
    let summary = summaries
        .iter()
        .max_by_key(|s| s.package_id().version())
        .unwrap();

    let package_id = summary.package_id();

    let package = registry.get(&[package_id])?;

    let package = package.get_one(package_id)?;

    pretty_view(&package, config)?;

    Ok(())
}

fn pretty_view(krate: &Package, config: &Config) -> CargoResult<()> {
    let summary = krate.manifest().summary();
    let package_id = summary.package_id();
    let manmeta = krate.manifest().metadata();

    config.shell().write_stdout("\n", &NOP)?;
    config.shell().write_stdout(
        format!(
            "{name}@{version}",
            name = package_id.name(),
            version = package_id.version()
        ),
        &GOOD,
    )?;
    config.shell().write_stdout(" | ", &NOP)?;
    match manmeta.license {
        Some(ref license) => {
            config
                .shell()
                .write_stdout(format!("{license}", license = license), &GOOD)?;
        }
        None => {
            config.shell().write_stdout("No license", &WARN)?;
        }
    }
    config.shell().write_stdout(" | ", &NOP)?;
    config.shell().write_stdout("deps: ", &NOP)?;
    let deps = summary.dependencies().len();
    config
        .shell()
        .write_stdout(deps, &AnsiColor::Cyan.on_default().effects(Effects::BOLD))?;

    config.shell().write_stdout("\n", &NOP)?;

    Ok(())
}
