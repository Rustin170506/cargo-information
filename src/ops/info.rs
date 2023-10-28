use std::io::Write;

use crate::ops::style::{CYAN, GREEN, YELLOW};
use cargo::core::registry::PackageRegistry;
use cargo::core::{Dependency, Package, QueryKind, Registry, SourceId, Summary, Target};
use cargo::{CargoResult, Config};

pub fn info(spec: &str, config: &Config) -> CargoResult<()> {
    let source_id = SourceId::crates_io(config)?;
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

    pretty_view(&package, &summaries, stdout)?;

    Ok(())
}

fn pretty_view(krate: &Package, summaries: &[Summary], stdout: &mut dyn Write) -> CargoResult<()> {
    let summary = krate.manifest().summary();
    let package_id = summary.package_id();
    let manmeta = krate.manifest().metadata();

    let green = GREEN.render();
    let yellow = YELLOW.render();
    let cyan = CYAN.render();
    let reset = anstyle::Reset.render();

    writeln!(stdout)?;
    write!(
        stdout,
        "{green}{name}{reset}@{green}{version}{reset}",
        name = package_id.name(),
        version = package_id.version()
    )?;
    write!(stdout, " | ")?;
    match manmeta.license {
        Some(ref license) => {
            write!(stdout, "{green}{license}{reset}", license = license)?;
        }
        None => {
            write!(stdout, "{yellow}No license{reset}")?;
        }
    }
    write!(stdout, " | ")?;
    write!(stdout, "deps: ")?;
    let deps = summary.dependencies().len();
    write!(stdout, "{cyan}{deps}{reset}")?;

    write!(stdout, " | ")?;
    write!(stdout, "versions: ")?;
    write!(stdout, "{yellow}{len}{reset}", len = summaries.len())?;
    writeln!(stdout)?;

    if let Some(ref description) = manmeta.description {
        writeln!(
            stdout,
            "{cyan}{description}{reset}",
            description = description.trim_end()
        )?;
    }
    writeln!(stdout)?;

    if let Some(ref homepage) = manmeta.homepage {
        write!(stdout, "Homepage: ")?;
        writeln!(stdout, "{cyan}{homepage}{reset}", homepage = homepage)?;
    }

    if let Some(ref repository) = manmeta.repository {
        write!(stdout, "Repository: ")?;
        writeln!(stdout, "{cyan}{repository}{reset}", repository = repository)?;
    }

    if let Some(ref documentation) = manmeta.documentation {
        write!(stdout, "Documentation: ")?;
        writeln!(
            stdout,
            "{cyan}{documentation}{reset}",
            documentation = documentation
        )?;
    }

    writeln!(stdout)?;

    if let Some(library) = krate.library() {
        write!(stdout, "lib: ")?;
        writeln!(stdout, "{cyan}{name}{reset}", name = library.name())?;
    }

    let binaries = krate
        .targets()
        .iter()
        .filter(|t| t.is_bin())
        .collect::<Vec<&Target>>();

    if !binaries.is_empty() {
        write!(stdout, "bin: ")?;
        for binary in binaries {
            write!(stdout, "{cyan}{name}{reset}", name = binary.name())?;
            write!(stdout, " ")?;
        }
        writeln!(stdout)?;
    }

    writeln!(stdout)?;

    Ok(())
}
