use std::io::Write;

use cargo::{
    core::SourceId,
    core::{dependency::DepKind, Dependency, FeatureMap, Package, PackageId, Summary},
    CargoResult, Config,
};

use super::style::{ERROR, HEADER, LITERAL, NOP, NOTE, WARN};

// Pretty print the package information.
pub(super) fn pretty_view(
    package: &Package,
    summaries: &[Summary],
    owners: &Option<Vec<String>>,
    suggest_cargo_tree_command: bool,
    config: &Config,
    stdout: &mut dyn Write,
) -> CargoResult<()> {
    let summary = package.manifest().summary();
    let package_id = summary.package_id();
    let metadata = package.manifest().metadata();
    let header = HEADER.render();
    let error = ERROR.render();
    let warn = WARN.render();
    let note = NOTE.render();
    let reset = anstyle::Reset.render();

    write!(stdout, "{header}{}{reset}", package_id.name())?;
    if !metadata.keywords.is_empty() {
        write!(stdout, " {note}#{}{reset}", metadata.keywords.join(" #"))?;
    }
    writeln!(stdout)?;
    if let Some(ref description) = metadata.description {
        writeln!(stdout, "{}", description.trim_end())?;
    }
    write!(stdout, "{header}version:{reset} {}", package_id.version())?;
    // Add a warning message to stdout if the following conditions are met:
    // 1. The package version is not the latest available version.
    // 2. The package source is not crates.io.
    match (
        summaries.iter().max_by_key(|s| s.version()),
        summary.source_id().is_crates_io(),
    ) {
        (Some(latest), false) if latest.version() != package_id.version() => {
            write!(
                stdout,
                " {warn}(latest {} {note}from {}{warn}){reset}",
                latest.version(),
                pretty_source(summary.source_id(), config)
            )?;
        }
        (Some(latest), true) if latest.version() != package_id.version() => {
            write!(stdout, " {warn}(latest {}){reset}", latest.version(),)?;
        }
        (_, false) => {
            write!(
                stdout,
                " {note}(from {}){reset}",
                pretty_source(summary.source_id(), config)
            )?;
        }
        (_, true) => {}
    }
    writeln!(stdout)?;
    writeln!(
        stdout,
        "{header}license:{reset} {}",
        metadata
            .license
            .clone()
            .unwrap_or_else(|| format!("{error}unknown{reset}"))
    )?;
    // TODO: color MSRV as a warning if newer than either the "workspace" MSRV or `rustc --version`
    writeln!(
        stdout,
        "{header}rust-version:{reset} {}",
        metadata
            .rust_version
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("{warn}unknown{reset}"))
    )?;
    if let Some(ref link) = metadata.documentation.clone().or_else(|| {
        summary.source_id().is_crates_io().then(|| {
            format!(
                "https://docs.rs/{name}/{version}",
                name = package_id.name(),
                version = package_id.version()
            )
        })
    }) {
        writeln!(stdout, "{header}documentation:{reset} {link}")?;
    }
    if let Some(ref link) = metadata.homepage {
        writeln!(stdout, "{header}homepage:{reset} {link}")?;
    }
    if let Some(ref link) = metadata.repository {
        writeln!(stdout, "{header}repository:{reset} {link}")?;
    }

    pretty_features(summary.features(), stdout)?;

    pretty_deps(package, stdout)?;

    if let Some(owners) = owners {
        pretty_owners(owners, stdout)?;
    }

    if suggest_cargo_tree_command {
        suggest_cargo_tree(package_id, stdout)?;
    }

    Ok(())
}

fn pretty_source(source: SourceId, config: &Config) -> String {
    if let Some(relpath) = source
        .local_path()
        .and_then(|path| pathdiff::diff_paths(path, config.cwd()))
    {
        let path = std::path::Path::new(".").join(relpath);
        path.display().to_string()
    } else {
        source.to_string()
    }
}

fn pretty_deps(package: &Package, stdout: &mut dyn Write) -> CargoResult<()> {
    let header = HEADER.render();
    let reset = anstyle::Reset.render();

    let dependencies = package
        .dependencies()
        .iter()
        .filter(|d| d.kind() == DepKind::Normal)
        .collect::<Vec<_>>();
    if !dependencies.is_empty() {
        writeln!(stdout, "{header}dependencies:{reset}")?;
        print_deps(dependencies, stdout)?;
    }

    let build_dependencies = package
        .dependencies()
        .iter()
        .filter(|d| d.kind() == DepKind::Build)
        .collect::<Vec<_>>();
    if !build_dependencies.is_empty() {
        writeln!(stdout, "{header}build-dependencies:{reset}")?;
        print_deps(build_dependencies, stdout)?;
    }

    Ok(())
}

fn print_deps(dependencies: Vec<&Dependency>, stdout: &mut dyn Write) -> Result<(), anyhow::Error> {
    for dependency in dependencies {
        let style = if dependency.is_optional() {
            anstyle::Style::new() | anstyle::Effects::DIMMED
        } else {
            Default::default()
        }
        .render();
        let reset = anstyle::Reset.render();
        writeln!(
            stdout,
            "  {style}{}@{}{reset}",
            dependency.package_name(),
            pretty_req(dependency.version_req())
        )?;
    }
    Ok(())
}

fn pretty_req(req: &cargo::util::OptVersionReq) -> String {
    let mut rendered = req.to_string();
    let strip_prefix = match req {
        cargo::util::OptVersionReq::Any => false,
        cargo::util::OptVersionReq::Req(req) | cargo::util::OptVersionReq::Locked(_, req) => {
            req.comparators.len() == 1 && rendered.starts_with('^')
        }
    };
    if strip_prefix {
        rendered.remove(0);
        rendered
    } else {
        rendered
    }
}

fn pretty_features(features: &FeatureMap, stdout: &mut dyn Write) -> CargoResult<()> {
    let header = HEADER.render();
    let enabled = LITERAL.render();
    let disabled = NOP.render();
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

    writeln!(stdout, "{header}features:{reset}")?;

    let default_feature = cargo::util::interning::InternedString::new("default");
    let mut root_activated = Vec::new();
    if features.iter().any(|(name, _)| *name == default_feature) {
        root_activated.push(default_feature);
    }

    let mut remaining = features.clone();
    for root in root_activated {
        let mut activated = vec![root];
        while let Some(current) = activated.pop() {
            let Some(current_activated) = remaining.remove(&current) else {
                continue;
            };
            writeln!(
                stdout,
                "  {enabled}{current: <margin$}{reset} = [{features}]",
                features = current_activated
                    .iter()
                    .map(|s| format!("{enabled}{s}{reset}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
            activated.extend(current_activated.iter().rev().filter_map(|f| match f {
                cargo::core::FeatureValue::Feature(name) => Some(name),
                cargo::core::FeatureValue::Dep { .. }
                | cargo::core::FeatureValue::DepFeature { .. } => None,
            }));
        }
    }

    let mut activated = remaining.keys().rev().cloned().collect::<Vec<_>>();
    while let Some(current) = activated.pop() {
        let Some(current_activated) = remaining.remove(&current) else {
            continue;
        };
        writeln!(
            stdout,
            "  {disabled}{current: <margin$}{reset} = [{features}]",
            features = current_activated
                .iter()
                .map(|s| format!("{disabled}{s}{reset}"))
                .collect::<Vec<String>>()
                .join(", ")
        )?;
        activated.extend(current_activated.iter().rev().filter_map(|f| match f {
            cargo::core::FeatureValue::Feature(name) => Some(name),
            cargo::core::FeatureValue::Dep { .. }
            | cargo::core::FeatureValue::DepFeature { .. } => None,
        }));
    }

    Ok(())
}

fn pretty_owners(owners: &Vec<String>, stdout: &mut dyn Write) -> CargoResult<()> {
    let header = HEADER.render();
    let reset = anstyle::Reset.render();

    if !owners.is_empty() {
        writeln!(stdout, "{header}owners:{reset}",)?;
        for owner in owners {
            writeln!(stdout, "  {}", owner)?;
        }
    }

    Ok(())
}

// Suggest the cargo tree command to view the dependency tree.
fn suggest_cargo_tree(package_id: PackageId, stdout: &mut dyn Write) -> CargoResult<()> {
    let literal = LITERAL.render();
    let reset = anstyle::Reset.render();

    note(format_args!(
        "to see how you depend on {name}, run `{literal}cargo tree --package {name}@{version} --invert{reset}`",
        name = package_id.name(),
        version = package_id.version(),
    ), stdout)
}

pub(super) fn note(msg: impl std::fmt::Display, stdout: &mut dyn Write) -> CargoResult<()> {
    let note = NOTE.render();
    let bold = (anstyle::Style::new() | anstyle::Effects::BOLD).render();
    let reset = anstyle::Reset.render();

    writeln!(stdout, "{note}note{reset}{bold}:{reset} {msg}",)?;

    Ok(())
}
