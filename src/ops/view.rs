use std::io::Write;

use cargo::{
    core::{
        dependency::DepKind, shell::Verbosity, Dependency, FeatureMap, Package, PackageId, SourceId,
    },
    sources::IndexSummary,
    CargoResult, Config,
};

use super::style::{ERROR, HEADER, LITERAL, NOP, NOTE, WARN};

// Pretty print the package information.
pub(super) fn pretty_view(
    package: &Package,
    summaries: &[IndexSummary],
    owners: &Option<Vec<String>>,
    suggest_cargo_tree_command: bool,
    config: &Config,
) -> CargoResult<()> {
    let summary = package.manifest().summary();
    let package_id = summary.package_id();
    let metadata = package.manifest().metadata();
    let header = HEADER;
    let error = ERROR;
    let warn = WARN;
    let note = NOTE;

    let mut shell = config.shell();
    let verbosity = shell.verbosity();
    let stdout = shell.out();
    write!(stdout, "{header}{}{header:#}", package_id.name())?;
    if !metadata.keywords.is_empty() {
        write!(stdout, " {note}#{}{note:#}", metadata.keywords.join(" #"))?;
    }
    writeln!(stdout)?;
    if let Some(ref description) = metadata.description {
        writeln!(stdout, "{}", description.trim_end())?;
    }
    write!(
        stdout,
        "{header}version:{header:#} {}",
        package_id.version()
    )?;
    // Add a warning message to stdout if the following conditions are met:
    // 1. The package version is not the latest available version.
    // 2. The package source is not crates.io.
    match (
        summaries.iter().max_by_key(|s| s.as_summary().version()),
        summary.source_id().is_crates_io(),
    ) {
        (Some(latest), false) if latest.as_summary().version() != package_id.version() => {
            write!(
                stdout,
                " {warn}(latest {} {warn:#}{note}from {}{note:#}{warn}){warn:#}",
                latest.as_summary().version(),
                pretty_source(summary.source_id(), config)
            )?;
        }
        (Some(latest), true) if latest.as_summary().version() != package_id.version() => {
            write!(
                stdout,
                " {warn}(latest {}){warn:#}",
                latest.as_summary().version(),
            )?;
        }
        (_, false) => {
            write!(
                stdout,
                " {note}(from {}){note:#}",
                pretty_source(summary.source_id(), config)
            )?;
        }
        (_, true) => {}
    }
    writeln!(stdout)?;
    writeln!(
        stdout,
        "{header}license:{header:#} {}",
        metadata
            .license
            .clone()
            .unwrap_or_else(|| format!("{error}unknown{error:#}"))
    )?;
    // TODO: color MSRV as a warning if newer than either the "workspace" MSRV or `rustc --version`
    writeln!(
        stdout,
        "{header}rust-version:{header:#} {}",
        metadata
            .rust_version
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("{warn}unknown{warn:#}"))
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
        writeln!(stdout, "{header}documentation:{header:#} {link}")?;
    }
    if let Some(ref link) = metadata.homepage {
        writeln!(stdout, "{header}homepage:{header:#} {link}")?;
    }
    if let Some(ref link) = metadata.repository {
        writeln!(stdout, "{header}repository:{header:#} {link}")?;
    }
    // Only print the crates.io link if the package is from crates.io.
    if summary.source_id().is_crates_io() {
        writeln!(
            stdout,
            "{header}crates.io:{header:#} https://crates.io/crates/{}/{}",
            package_id.name(),
            package_id.version()
        )?;
    }

    pretty_features(summary.features(), verbosity, stdout)?;

    pretty_deps(package, verbosity, stdout, config)?;

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

fn pretty_deps(
    package: &Package,
    verbosity: Verbosity,
    stdout: &mut dyn Write,
    config: &Config,
) -> CargoResult<()> {
    match verbosity {
        Verbosity::Quiet | Verbosity::Normal => {
            return Ok(());
        }
        Verbosity::Verbose => {}
    }

    let header = HEADER;

    let dependencies = package
        .dependencies()
        .iter()
        .filter(|d| d.kind() == DepKind::Normal)
        .collect::<Vec<_>>();
    if !dependencies.is_empty() {
        writeln!(stdout, "{header}dependencies:{header:#}")?;
        print_deps(dependencies, stdout, config)?;
    }

    let build_dependencies = package
        .dependencies()
        .iter()
        .filter(|d| d.kind() == DepKind::Build)
        .collect::<Vec<_>>();
    if !build_dependencies.is_empty() {
        writeln!(stdout, "{header}build-dependencies:{header:#}")?;
        print_deps(build_dependencies, stdout, config)?;
    }

    Ok(())
}

fn print_deps(
    dependencies: Vec<&Dependency>,
    stdout: &mut dyn Write,
    config: &Config,
) -> Result<(), anyhow::Error> {
    for dependency in dependencies {
        let style = if dependency.is_optional() {
            anstyle::Style::new() | anstyle::Effects::DIMMED
        } else {
            Default::default()
        };
        // 1. Only print the version requirement if it is a registry dependency.
        // 2. Only print the source if it is not a registry dependency.
        // For example: `bar (./crates/bar)` or `bar@=1.2.3`.
        let (req, source) = if dependency.source_id().is_registry() {
            (
                format!("@{}", pretty_req(dependency.version_req())),
                String::new(),
            )
        } else {
            (
                String::new(),
                format!(" ({})", pretty_source(dependency.source_id(), config)),
            )
        };

        writeln!(
            stdout,
            "  {style}{}{}{}{style:#}",
            dependency.package_name(),
            req,
            source
        )?;
    }
    Ok(())
}

fn pretty_req(req: &cargo::util::OptVersionReq) -> String {
    let mut rendered = req.to_string();
    let strip_prefix = match req {
        cargo::util::OptVersionReq::Any => false,
        cargo::util::OptVersionReq::Req(req)
        | cargo::util::OptVersionReq::Locked(_, req)
        | cargo::util::OptVersionReq::UpdatePrecise(_, req) => {
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

fn pretty_features(
    features: &FeatureMap,
    verbosity: Verbosity,
    stdout: &mut dyn Write,
) -> CargoResult<()> {
    let header = HEADER;
    let enabled = LITERAL;
    let disabled = NOP;
    let summary = anstyle::Style::new() | anstyle::Effects::ITALIC;

    // If there are no features, return early.
    let margin = features
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or_default();
    if margin == 0 {
        return Ok(());
    }

    writeln!(stdout, "{header}features:{header:#}")?;

    let default_feature = cargo::util::interning::InternedString::new("default");
    let mut activated_queue = Vec::new();
    if features.iter().any(|(name, _)| *name == default_feature) {
        activated_queue.push(default_feature);
    }

    let mut activated = vec![];
    let mut remaining = features.clone();
    while let Some(current) = activated_queue.pop() {
        let Some(current_activated) = remaining.remove(&current) else {
            continue;
        };
        activated_queue.extend(current_activated.iter().rev().filter_map(|f| match f {
            cargo::core::FeatureValue::Feature(name) => Some(name),
            cargo::core::FeatureValue::Dep { .. }
            | cargo::core::FeatureValue::DepFeature { .. } => None,
        }));
        activated.push((current, current_activated));
    }

    const MAX_FEATURE_PRINTS: usize = 30;
    let total_activated = activated.len();
    let total_deactivated = remaining.len();
    let show_all = match verbosity {
        Verbosity::Quiet | Verbosity::Normal => false,
        Verbosity::Verbose => true,
    };
    if total_activated <= MAX_FEATURE_PRINTS || show_all {
        activated.sort_by_key(|(name, _)| {
            // sort `default` first
            if name == "default" {
                None
            } else {
                Some(name.to_owned())
            }
        });

        for (current, current_activated) in activated {
            writeln!(
                stdout,
                "  {enabled}{current: <margin$}{enabled:#} = [{features}]",
                features = current_activated
                    .iter()
                    .map(|s| format!("{enabled}{s}{enabled:#}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
        }
    } else {
        writeln!(
            stdout,
            "  {summary}{total_activated} activated features{summary:#}",
        )?;
    }

    if (total_activated + total_deactivated) <= MAX_FEATURE_PRINTS || show_all {
        for (current, current_activated) in remaining {
            writeln!(
                stdout,
                "  {disabled}{current: <margin$}{disabled:#} = [{features}]",
                features = current_activated
                    .iter()
                    .map(|s| format!("{disabled}{s}{disabled:#}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
        }
    } else {
        writeln!(
            stdout,
            "  {summary}{total_deactivated} deactivated features{summary:#}",
        )?;
    }

    Ok(())
}

fn pretty_owners(owners: &Vec<String>, stdout: &mut dyn Write) -> CargoResult<()> {
    let header = HEADER;

    if !owners.is_empty() {
        writeln!(stdout, "{header}owners:{header:#}",)?;
        for owner in owners {
            writeln!(stdout, "  {}", owner)?;
        }
    }

    Ok(())
}

// Suggest the cargo tree command to view the dependency tree.
fn suggest_cargo_tree(package_id: PackageId, stdout: &mut dyn Write) -> CargoResult<()> {
    let literal = LITERAL;

    note(format_args!(
        "to see how you depend on {name}, run `{literal}cargo tree --invert --package {name}@{version}{literal:#}`",
        name = package_id.name(),
        version = package_id.version(),
    ), stdout)
}

pub(super) fn note(msg: impl std::fmt::Display, stdout: &mut dyn Write) -> CargoResult<()> {
    let note = NOTE;
    let bold = anstyle::Style::new() | anstyle::Effects::BOLD;

    writeln!(stdout, "{note}note{note:#}{bold}:{bold:#} {msg}",)?;

    Ok(())
}
