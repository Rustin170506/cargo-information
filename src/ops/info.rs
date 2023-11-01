use std::io::Write;

use crate::ops::style::{CYAN, GREEN, YELLOW};
use anyhow::Ok;
use cargo::core::dependency::DepKind;
use cargo::core::registry::PackageRegistry;
use cargo::core::{
    Dependency, FeatureMap, Package, QueryKind, Registry, SourceId, Summary, Target,
};
use cargo::{CargoResult, Config};

/// Check the information about a package.
pub fn info(spec: &str, config: &Config) -> CargoResult<()> {
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

// Pretty print the package information.
fn pretty_view(
    package: &Package,
    summaries: &[Summary],
    stdout: &mut dyn Write,
) -> CargoResult<()> {
    let summary = package.manifest().summary();
    let package_id = summary.package_id();
    let metadata = package.manifest().metadata();

    let green = GREEN.render();
    let yellow = YELLOW.render();
    let cyan = CYAN.render();
    let reset = anstyle::Reset.render();

    // Basic information.
    writeln!(stdout)?;
    write!(
        stdout,
        "{green}{name}{reset}@{green}{version}{reset}",
        name = package_id.name(),
        version = package_id.version()
    )?;
    write!(stdout, " | ")?;
    match metadata.license {
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
    if let Some(rust_version) = &metadata.rust_version {
        write!(stdout, " | ")?;
        write!(stdout, "rust: ")?;
        write!(stdout, "{yellow}{rust_version}{reset}")?;
    }
    writeln!(stdout)?;

    // Keywords.
    if !metadata.keywords.is_empty() {
        write!(stdout, "keywords: ")?;
        writeln!(
            stdout,
            "{cyan}#{keywords}{reset}",
            keywords = metadata.keywords.join("  #")
        )?;
        writeln!(stdout)?;
    }

    // Description and links.
    if let Some(ref description) = metadata.description {
        writeln!(
            stdout,
            "{description}",
            description = description.trim_end()
        )?;
        writeln!(stdout)?;
    }
    if let Some(ref homepage) = metadata.homepage {
        write!(stdout, "Homepage: ")?;
        writeln!(stdout, "{cyan}{homepage}{reset}")?;
    }
    if let Some(ref repository) = metadata.repository {
        write!(stdout, "Repository: ")?;
        writeln!(stdout, "{cyan}{repository}{reset}")?;
    }
    if let Some(ref documentation) = metadata.documentation {
        write!(stdout, "Documentation: ")?;
        writeln!(stdout, "{cyan}{documentation}{reset}")?;
    }
    writeln!(stdout)?;

    // Kind.
    if let Some(library) = package.library() {
        write!(stdout, "lib: ")?;
        writeln!(stdout, "{cyan}{name}{reset}", name = library.name())?;
    }
    let binaries = package
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

    pretty_deps(package, stdout)?;

    pretty_features(summary.features(), stdout)?;

    Ok(())
}

fn pretty_deps(package: &Package, stdout: &mut dyn Write) -> CargoResult<()> {
    let yellow = YELLOW.render();
    let reset = anstyle::Reset.render();

    let dependencies = package
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

    let dev_dependencies = package
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

    let build_dependencies = package
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

    // If there are no features, return early.
    let margin = features
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or_default();
    if margin == 0 {
        return Ok(());
    }

    writeln!(stdout, "features:")?;

    // Find the default features.
    const DEFAULT_FEATURE_NAME: &str = "default";
    let default_features = features
        .iter()
        .find(|(name, _)| name.as_str() == DEFAULT_FEATURE_NAME)
        .map(|f| f.1.iter().map(|f| f.to_string()).collect::<Vec<String>>())
        .unwrap();
    write!(stdout, "{cyan}")?;
    write!(stdout, "{DEFAULT_FEATURE_NAME: <margin$}")?;
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
        if name.as_str() == DEFAULT_FEATURE_NAME {
            continue;
        }
        // If the feature is a default feature, color it yellow.
        if default_features.contains(&name.to_string()) {
            write!(stdout, "{yellow}")?;
            write!(stdout, "{name: <margin$}")?;
            write!(stdout, "{reset} = ")?;
        } else {
            write!(stdout, "{name: <margin$} = ")?;
        }
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
    Ok(())
}
