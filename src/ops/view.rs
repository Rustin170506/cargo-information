use std::io::Write;

use cargo::{
    core::{dependency::DepKind, FeatureMap, Package, Summary, Target},
    CargoResult,
};

use super::style::{CYAN, GREEN, YELLOW};

// Pretty print the package information.
pub(super) fn pretty_view(
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
    let deps = package.dependencies().len();
    write!(stdout, "{cyan}{deps}{reset}")?;
    write!(stdout, " | ")?;
    write!(stdout, "versions: ")?;
    write!(stdout, "{yellow}{len}{reset}", len = summaries.len())?;
    write!(stdout, " | ")?;
    write!(stdout, "edition: ")?;
    write!(
        stdout,
        "{yellow}{edition}{reset}",
        edition = package.manifest().edition()
    )?;
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

    pretty_authors(&metadata.authors, stdout)?;

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
    let default_features = &features
        .iter()
        .find(|(name, _)| name.as_str() == DEFAULT_FEATURE_NAME)
        .map(|f| f.1.iter().map(|f| f.to_string()).collect::<Vec<String>>());
    if default_features.is_some() {
        write!(stdout, "{cyan}")?;
        write!(stdout, "{DEFAULT_FEATURE_NAME: <margin$}")?;
        write!(stdout, "{reset} = ")?;
        writeln!(
            stdout,
            "[{features}]",
            features = default_features
                .as_ref()
                .unwrap()
                .iter()
                .map(|s| format!("{yellow}{s}{reset}"))
                .collect::<Vec<String>>()
                .join(", ")
        )?;
    }

    for (name, features) in features.iter() {
        if name.as_str() == DEFAULT_FEATURE_NAME {
            continue;
        }
        // If the feature is a default feature, color it yellow.
        if default_features.is_some()
            && default_features
                .as_ref()
                .unwrap()
                .contains(&name.to_string())
        {
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
    writeln!(stdout)?;

    Ok(())
}

fn pretty_authors(authors: &[String], stdout: &mut dyn Write) -> CargoResult<()> {
    let yellow = YELLOW.render();
    let reset = anstyle::Reset.render();

    if !authors.is_empty() {
        writeln!(stdout, "authors:")?;
        for author in authors {
            writeln!(stdout, "- {yellow}{author}{reset}")?;
        }
    }

    Ok(())
}
