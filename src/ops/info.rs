use std::collections::HashSet;
use std::path::PathBuf;
use std::task::Poll;

use anyhow::{bail, Context as _};
use cargo::core::registry::PackageRegistry;
use cargo::core::{Dependency, PackageId, PackageIdSpec, Registry, SourceId, Workspace};
use cargo::ops::RegistryOrIndex;
use cargo::sources::source::{QueryKind, Source};
use cargo::sources::{RegistrySource, SourceConfigMap};
use cargo::util::auth::{auth_token, AuthorizationErrorReason};
use cargo::util::cache_lock::CacheLockMode;
use cargo::util::command_prelude::root_manifest;
use cargo::util::network::http::http_handle;
use cargo::{ops, CargoResult, Config};
use cargo_credential::Operation;
use crates_io::Registry as CratesIoRegistry;
use crates_io::User;

use super::view::pretty_view;

pub fn info(
    spec: &PackageIdSpec,
    config: &Config,
    reg_or_index: Option<RegistryOrIndex>,
    ignore_rust_version: bool,
) -> CargoResult<()> {
    let mut registry = PackageRegistry::new(config)?;
    // Make sure we get the lock before we download anything.
    let _lock = config.acquire_package_cache_lock(CacheLockMode::DownloadExclusive)?;
    registry.lock_patches();

    // If we can find it in workspace, use it as a specific version.
    let root_manifest = root_manifest(None, config).ok();
    let ws = root_manifest
        .as_ref()
        .and_then(|root| Workspace::new(root, config).ok());
    let (mut package_id, is_member) = ws
        .as_ref()
        .and_then(|ws| ops::resolve_ws(ws).map(|(_, resolve)| (ws, resolve)).ok())
        .and_then(|(ws, resolve)| {
            let package_id = resolve
                .iter()
                .filter(|&p| spec.matches(p))
                .max_by_key(|&p| p.version());
            package_id.map(|pid| (Some(pid), ws.members().any(|p| p.package_id() == pid)))
        })
        .unwrap_or((None, false));
    let (use_package_source_id, source_ids) = get_source_id(config, reg_or_index, package_id)?;
    // If we don't use the package's source, we need to query the package ID from the specified registry.
    if !use_package_source_id {
        package_id = None;
    }

    validate_locked_and_frozen_options(package_id, config)?;

    let mut rustc_version = None;
    if !ignore_rust_version {
        let msrv_from_nearest_manifest_or_ws =
            try_get_msrv_from_nearest_manifest_or_ws(&root_manifest, ws.as_ref());
        // If the workspace does not have a specific Rust version,
        // or if the command is not called within the workspace, then fallback to the global Rust version.
        rustc_version = Some(match msrv_from_nearest_manifest_or_ws {
            Some(msrv) => msrv,
            None => {
                let current_rustc = config.load_global_rustc(ws.as_ref())?.version;
                // Remove any pre-release identifiers for easier comparison.
                // Otherwise, the MSRV check will fail if the current Rust version is a nightly or beta version.
                semver::Version::new(
                    current_rustc.major,
                    current_rustc.minor,
                    current_rustc.patch,
                )
            }
        });
    }

    // Only suggest cargo tree command when the package is not a workspace member.
    // For workspace members, `cargo tree --package <SPEC> --invert` is useless. It only prints itself.
    let suggest_cargo_tree_command = package_id.is_some() && !is_member;
    query_and_pretty_view(
        spec,
        package_id,
        config,
        registry,
        source_ids,
        rustc_version,
        suggest_cargo_tree_command,
    )
}

// Query the package registry and pretty print the result.
// If package_id is None, find the latest version.
fn query_and_pretty_view(
    spec: &PackageIdSpec,
    package_id: Option<PackageId>,
    config: &Config,
    mut registry: PackageRegistry,
    source_ids: RegistrySourceIds,
    rustc_version: Option<semver::Version>,
    suggest_cargo_tree_command: bool,
) -> CargoResult<()> {
    // Query without version requirement to get all index summaries.
    let dep = Dependency::parse(spec.name(), None, source_ids.original)?;
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
            let summary = summaries
                .iter()
                .filter(|s| spec.matches(s.package_id()))
                .max_by(|s1, s2| {
                    // Check the MSRV compatibility.
                    let s1_matches = rustc_version
                        .as_ref()
                        .and_then(|rustc_version| {
                            s1.rust_version()
                                .map(|v| v.to_caret_req().matches(rustc_version))
                        })
                        .unwrap_or(false);
                    let s2_matches = rustc_version
                        .as_ref()
                        .and_then(|rustc_version| {
                            s2.rust_version()
                                .map(|v| v.to_caret_req().matches(rustc_version))
                        })
                        .unwrap_or(false);
                    // MSRV compatible version is preferred.
                    match (s1_matches, s2_matches) {
                        (true, false) => std::cmp::Ordering::Greater,
                        (false, true) => std::cmp::Ordering::Less,
                        // If both summaries match the current Rust version or neither do, try to
                        // pick the latest version.
                        _ => s1.package_id().version().cmp(s2.package_id().version()),
                    }
                });
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
    let owners = try_list_owners(config, source_ids, package_id.name().as_str())?;
    let mut shell = config.shell();
    let stdout = shell.out();
    pretty_view(
        package,
        &summaries,
        &owners,
        suggest_cargo_tree_command,
        config,
        stdout,
    )?;

    Ok(())
}

// Try to list the login and name of all owners of a crate.
fn try_list_owners(
    config: &Config,
    source_ids: RegistrySourceIds,
    package_name: &str,
) -> CargoResult<Option<Vec<String>>> {
    // Only remote registries support listing owners.
    if !source_ids.original.is_remote_registry() {
        return Ok(None);
    }
    let registry = api_registry(config, source_ids)?;
    match registry {
        Some(mut registry) => {
            let owners = registry.list_owners(package_name)?;
            let names = owners.iter().map(get_username).collect();
            Ok(Some(names))
        }
        None => Ok(None),
    }
}

fn get_username(u: &User) -> String {
    format!(
        "{}{}",
        u.login,
        u.name
            .as_ref()
            .map(|name| format!(" ({})", name))
            .unwrap_or_default(),
    )
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
    replacement: SourceId,
}

fn get_source_id(
    config: &Config,
    reg_or_index: Option<RegistryOrIndex>,
    package_id: Option<PackageId>,
) -> CargoResult<(bool, RegistrySourceIds)> {
    let (use_package_source_id, sid) = match (&reg_or_index, package_id) {
        (None, Some(package_id)) => (true, package_id.source_id()),
        (None, None) => (false, SourceId::crates_io(config)?),
        (Some(RegistryOrIndex::Index(url)), None) => (false, SourceId::for_registry(url)?),
        (Some(RegistryOrIndex::Registry(r)), None) => (false, SourceId::alt_registry(config, r)?),
        (Some(reg_or_index), Some(package_id)) => {
            let sid = match reg_or_index {
                RegistryOrIndex::Index(url) => SourceId::for_registry(url)?,
                RegistryOrIndex::Registry(r) => SourceId::alt_registry(config, r)?,
            };
            let package_source_id = package_id.source_id();
            // Same registry, use the package's source.
            if sid == package_source_id {
                (true, sid)
            } else {
                let pkg_source_replacement_sid = SourceConfigMap::new(config)?
                    .load(package_source_id, &HashSet::new())?
                    .replaced_source_id();
                // Use the package's source if the specified registry is a replacement for the package's source.
                if pkg_source_replacement_sid == sid {
                    (true, package_source_id)
                } else {
                    (false, sid)
                }
            }
        }
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
        Ok((
            use_package_source_id,
            RegistrySourceIds {
                original: sid,
                replacement: builtin_replacement_sid,
            },
        ))
    }
}

// Try to get the crates.io registry which is used to access the registry API.
// If the user is not logged in, the function will return None.
fn api_registry(
    config: &Config,
    source_ids: RegistrySourceIds,
) -> CargoResult<Option<CratesIoRegistry>> {
    let cfg = {
        let mut src = RegistrySource::remote(source_ids.replacement, &HashSet::new(), config)?;
        let cfg = loop {
            match src.config()? {
                Poll::Pending => src
                    .block_until_ready()
                    .with_context(|| format!("failed to update {}", source_ids.replacement))?,
                Poll::Ready(cfg) => break cfg,
            }
        };
        cfg.expect("remote registries must have config")
    };
    // This should only happen if the user has a custom registry configured.
    // Some registries may not have API support.
    let api_host = match cfg.api {
        Some(api_host) => api_host,
        None => return Ok(None),
    };
    let token = match auth_token(
        config,
        &source_ids.original,
        None,
        Operation::Read,
        vec![],
        false,
    ) {
        Ok(token) => Some(token),
        Err(err) => {
            // If the token is missing, it means the user is not logged in.
            // We don't want to show an error in this case.
            if err.to_string().contains(
                (AuthorizationErrorReason::TokenMissing)
                    .to_string()
                    .as_str(),
            ) {
                return Ok(None);
            }
            return Err(err);
        }
    };

    let handle = http_handle(config)?;
    Ok(Some(CratesIoRegistry::new_handle(
        api_host,
        token,
        handle,
        cfg.auth_required,
    )))
}

fn validate_locked_and_frozen_options(
    package_id: Option<PackageId>,
    config: &Config,
) -> Result<(), anyhow::Error> {
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
    Ok(())
}

fn try_get_msrv_from_nearest_manifest_or_ws(
    nearest_manifest: &Option<PathBuf>,
    ws: Option<&Workspace>,
) -> Option<semver::Version> {
    // Try to get the MSRV from the nearest manifest.
    let rust_version = ws
        .as_ref()
        .and_then(|ws| {
            nearest_manifest
                .as_ref()
                .and_then(|path| ws.members().find(|p| p.manifest_path() == path))
        })
        .and_then(|p| p.rust_version()?.to_version());
    // If the nearest manifest does not have a specific Rust version, try to get it from the workspace.
    rust_version.or_else(|| ws.and_then(|ws| ws.rust_version()?.to_version()))
}
