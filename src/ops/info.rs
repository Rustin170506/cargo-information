use std::collections::HashSet;

use crate::ops::style::{CYAN, GREEN, NOP, YELLOW};
use cargo::core::registry::PackageRegistry;
use cargo::core::{Dependency, Package, SourceId, Summary, Target};
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

    // Find the latest version.
    let summary = summaries
        .iter()
        .max_by_key(|s| s.package_id().version())
        .unwrap();

    let package_id = summary.package_id();

    let package = registry.get(&[package_id])?;

    let package = package.get_one(package_id)?;

    pretty_view(&package, &summaries, config)?;

    Ok(())
}

fn pretty_view(krate: &Package, summaries: &[Summary], config: &Config) -> CargoResult<()> {
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
        &GREEN,
    )?;
    config.shell().write_stdout(" | ", &NOP)?;
    match manmeta.license {
        Some(ref license) => {
            config
                .shell()
                .write_stdout(format!("{license}", license = license), &GREEN)?;
        }
        None => {
            config.shell().write_stdout("No license", &YELLOW)?;
        }
    }
    config.shell().write_stdout(" | ", &NOP)?;
    config.shell().write_stdout("deps: ", &NOP)?;
    let deps = summary.dependencies().len();
    config.shell().write_stdout(deps, &CYAN)?;

    config.shell().write_stdout(" | ", &NOP)?;
    config.shell().write_stdout("versions: ", &NOP)?;
    config.shell().write_stdout(summaries.len(), &YELLOW)?;

    config.shell().write_stdout("\n", &NOP)?;

    if let Some(ref description) = manmeta.description {
        config.shell().write_stdout(description.trim_end(), &NOP)?;
        config.shell().write_stdout("\n", &NOP)?;
    }

    if let Some(ref homepage) = manmeta.homepage {
        config.shell().write_stdout("Homepage: ", &NOP)?;
        config.shell().write_stdout(homepage, &CYAN)?;
        config.shell().write_stdout("\n", &NOP)?;
    }

    if let Some(ref repository) = manmeta.repository {
        config.shell().write_stdout("Repository: ", &NOP)?;
        config.shell().write_stdout(repository, &CYAN)?;
        config.shell().write_stdout("\n", &NOP)?;
    }

    if let Some(ref documentation) = manmeta.documentation {
        config.shell().write_stdout("Documentation: ", &NOP)?;
        config.shell().write_stdout(documentation, &CYAN)?;
        config.shell().write_stdout("\n", &NOP)?;
    }

    config.shell().write_stdout("\n", &NOP)?;

    if let Some(library) = krate.library() {
        config.shell().write_stdout("lib: ", &NOP)?;
        config.shell().write_stdout(library.name(), &CYAN)?;
        config.shell().write_stdout("\n", &NOP)?;
    }

    let binaries = krate
        .targets()
        .iter()
        .filter(|t| t.is_bin())
        .collect::<Vec<&Target>>();

    if !binaries.is_empty() {
        config.shell().write_stdout("bin: ", &NOP)?;
        for binary in binaries {
            config.shell().write_stdout(binary.name(), &CYAN)?;
            config.shell().write_stdout(" ", &NOP)?;
        }
        config.shell().write_stdout("\n", &NOP)?;
    }

    Ok(())
}
