use std::io::Write;

use crate::ops::style::{CYAN, GREEN, YELLOW};
use anyhow::Ok;
use cargo::core::dependency::DepKind;
use cargo::core::registry::PackageRegistry;
use cargo::core::{
    Dependency, FeatureMap, Package, QueryKind, Registry, SourceId, Summary, Target,
};
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

    pretty_view(package, &summaries, stdout)?;

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
            "{description}",
            description = description.trim_end()
        )?;
    }
    writeln!(stdout)?;

    if let Some(ref homepage) = manmeta.homepage {
        write!(stdout, "Homepage: ")?;
        writeln!(stdout, "{cyan}{homepage}{reset}")?;
    }

    if let Some(ref repository) = manmeta.repository {
        write!(stdout, "Repository: ")?;
        writeln!(stdout, "{cyan}{repository}{reset}")?;
    }

    if let Some(ref documentation) = manmeta.documentation {
        write!(stdout, "Documentation: ")?;
        writeln!(stdout, "{cyan}{documentation}{reset}")?;
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

    pretty_deps(krate, stdout)?;

    pretty_features(summary.features(), stdout)?;

    Ok(())
}

fn pretty_deps(krate: &Package, stdout: &mut dyn Write) -> CargoResult<()> {
    let yellow = YELLOW.render();
    let reset = anstyle::Reset.render();

    let dependencies = krate
        .dependencies()
        .iter()
        .filter(|d| d.kind() == DepKind::Normal)
        .map(|d| {
            format!(
                "{yellow}{name}{reset}: {version}",
                name = d.package_name(),
                version = d.version_req()
            )
        })
        .collect::<Vec<String>>();
    if !dependencies.is_empty() {
        writeln!(stdout, "dependencies:")?;
        print_deps(dependencies, stdout)?;
    }

    let dev_dependencies = krate
        .dependencies()
        .iter()
        .filter(|d| d.kind() == DepKind::Development)
        .map(|d| {
            format!(
                "{yellow}{name}{reset}: {version}",
                name = d.package_name(),
                version = d.version_req()
            )
        })
        .collect::<Vec<String>>();
    if !dev_dependencies.is_empty() {
        writeln!(stdout, "dev-dependencies:")?;
        print_deps(dev_dependencies, stdout)?;
    }

    let build_dependencies = krate
        .dependencies()
        .iter()
        .filter(|d| d.kind() == DepKind::Build)
        .map(|d| {
            format!(
                "{yellow}{name}{reset}: {version}",
                name = d.package_name(),
                version = d.version_req()
            )
        })
        .collect::<Vec<String>>();

    if !build_dependencies.is_empty() {
        writeln!(stdout, "build-dependencies:")?;
        print_deps(build_dependencies, stdout)?;
    }

    Ok(())
}

fn print_deps(dependencies: Vec<String>, stdout: &mut dyn Write) -> Result<(), anyhow::Error> {
    let margin = dependencies.iter().map(|d| d.len()).max().unwrap_or(0) + 2;
    let mut count = 0;
    for dep in &dependencies {
        if count + margin > 128 {
            writeln!(stdout)?;
            count = 0;
        }
        write!(stdout, "{dep: <margin$}")?;
        count += margin;
    }
    writeln!(stdout, "\n")?;
    Ok(())
}

fn pretty_features(features: &FeatureMap, stdout: &mut dyn Write) -> CargoResult<()> {
    let yellow = YELLOW.render();
    let cyan = CYAN.render();
    let reset = anstyle::Reset.render();

    writeln!(stdout, "features:")?;
    let margin = features
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or_default();
    if margin == 0 {
        return Ok(());
    }

    // Find the default features.
    let default_features = features
        .iter()
        .find(|(name, _)| name.as_str() == "default")
        .map(|f| f.1.iter().map(|f| f.to_string()).collect::<Vec<String>>())
        .unwrap();
    let default = "default".to_owned();
    write!(stdout, "{cyan}")?;
    write!(stdout, "{default: <margin$}")?;
    write!(stdout, "{reset} = ")?;
    writeln!(
        stdout,
        "[{features}]",
        features = default_features
            .iter()
            .map(|s| format!("{yellow}{s}{reset}"))
            .collect::<Vec<String>>()
            .join(", ")
    )?;
    for (name, features) in features.iter() {
        if name.as_str() == "default" {
            continue;
        }
        if default_features.contains(&name.to_string()) {
            write!(stdout, "{yellow}")?;
            write!(stdout, "{name: <margin$}")?;
            write!(stdout, "{reset} = ")?;
        } else {
            write!(stdout, "{name: <margin$} = ")?;
        }
        if !features.is_empty() {
            writeln!(
                stdout,
                "[{features}]",
                features = features
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
        }
    }
    Ok(())
}
