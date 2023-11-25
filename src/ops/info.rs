use std::collections::HashSet;

use anyhow::bail;
use cargo::core::registry::PackageRegistry;
use cargo::core::{Dependency, PackageId, Registry, SourceId, Summary, Workspace};
use cargo::ops::RegistryOrIndex;
use cargo::sources::source::QueryKind;
use cargo::sources::SourceConfigMap;
use cargo::util::cache_lock::CacheLockMode;
use cargo::util::command_prelude::root_manifest;
use cargo::{ops, CargoResult, Config};

use super::view::{pretty_view, suggest_cargo_tree};

pub fn info(spec: &str, config: &Config, reg_or_index: Option<RegistryOrIndex>) -> CargoResult<()> {
    let mut registry = PackageRegistry::new(config)?;
    // Make sure we get the lock before we download anything.
    let _lock = config.acquire_package_cache_lock(CacheLockMode::DownloadExclusive)?;
    registry.lock_patches();

    let mut package_id = None;
    // If we can find it in workspace, use it as a specific version.
    if let Ok(root) = root_manifest(None, config) {
        let ws = Workspace::new(&root, config)?;
        if let Some(resolve) = ops::load_pkg_lockfile(&ws)? {
            if let Ok(p) = resolve.query(spec) {
                package_id = Some(p)
            }
        }
    }

    let source_ids = get_source_id(config, reg_or_index, package_id)?;

    query_and_pretty_view(spec, package_id, config, registry, source_ids)
}

// Query the package registry and pretty print the result.
// If package_id is None, find the latest version.
fn query_and_pretty_view(
    spec: &str,
    package_id: Option<PackageId>,
    config: &Config,
    mut registry: PackageRegistry,
    source_ids: RegistrySourceIds,
) -> CargoResult<()> {
    let from_workspace = package_id.is_some();
    // Only in workspace, we can use --frozen or --locked.
    if !from_workspace {
        if config.locked() {
            anyhow::bail!("the option `--locked` can only be used within a workspace");
        }

        if config.frozen() {
            anyhow::bail!("the option `--frozen` can only be used within a workspace");
        }
    }

    // Query without version requirement to get all index summaries.
    let dep = Dependency::parse(spec, None, source_ids.original)?;
    let summaries = loop {
        // Exact to avoid returning all for path/git
        match registry.query_vec(&dep, QueryKind::Exact) {
            std::task::Poll::Ready(res) => {
                break res?;
            }
            std::task::Poll::Pending => registry.block_until_ready()?,
        }
    };

    let package_id = match package_id {
        Some(id) => id,
        None => {
            // Find the latest version.
            let summary = summaries.iter().max_by_key(|s| s.package_id().version());

            // If can not find the latest version, return an error.
            match summary {
                Some(summary) => summary.package_id(),
                None => {
                    anyhow::bail!(
                        "could not find `{}` in registry `{}`",
                        spec,
                        source_ids.original.url()
                    )
                }
            }
        }
    };

    let package = registry.get(&[package_id])?;
    let package = package.get_one(package_id)?;

    let mut shell = config.shell();
    let stdout = shell.out();

    let summaries: Vec<Summary> = summaries.iter().map(|s| s.as_summary().clone()).collect();
    pretty_view(package, &summaries, stdout)?;

    // Suggest the cargo tree command if the package is from workspace.
    if from_workspace {
        suggest_cargo_tree(package_id, stdout)?;
    }

    Ok(())
}

struct RegistrySourceIds {
    /// Use when looking up the auth token, or writing out `Cargo.lock`
    original: SourceId,
    /// Use when interacting with the source (querying / publishing , etc)
    ///
    /// The source for crates.io may be replaced by a built-in source for accessing crates.io with
    /// the sparse protocol, or a source for the testing framework (when the replace_crates_io
    /// function is used)
    ///
    /// User-defined source replacement is not applied.
    /// Note: This will be utilized when interfacing with the registry API.
    #[allow(dead_code)]
    replacement: SourceId,
}

fn get_source_id(
    config: &Config,
    reg_or_index: Option<RegistryOrIndex>,
    package_id: Option<PackageId>,
) -> CargoResult<RegistrySourceIds> {
    let sid = match (&reg_or_index, package_id) {
        (_, Some(package_id)) => package_id.source_id(),
        (None, None) => SourceId::crates_io(config)?,
        (Some(RegistryOrIndex::Index(url)), None) => SourceId::for_registry(url)?,
        (Some(RegistryOrIndex::Registry(r)), None) => SourceId::alt_registry(config, r)?,
    };

    // Load source replacements that are built-in to Cargo.
    let builtin_replacement_sid = SourceConfigMap::empty(config)?
        .load(sid, &HashSet::new())?
        .replaced_source_id();
    let replacement_sid = SourceConfigMap::new(config)?
        .load(sid, &HashSet::new())?
        .replaced_source_id();
    // Check if the user has configured source-replacement for the registry we are querying.
    if reg_or_index.is_none() && replacement_sid != builtin_replacement_sid {
        // Neither --registry nor --index was passed and the user has configured source-replacement.
        if let Some(replacement_name) = replacement_sid.alt_registry_key() {
            bail!("crates-io is replaced with remote registry {replacement_name};\ninclude `--registry {replacement_name}` or `--registry crates-io`");
        } else {
            bail!("crates-io is replaced with non-remote-registry source {replacement_sid};\ninclude `--registry crates-io` to use crates.io");
        }
    } else {
        Ok(RegistrySourceIds {
            original: sid,
            replacement: builtin_replacement_sid,
        })
    }
}
