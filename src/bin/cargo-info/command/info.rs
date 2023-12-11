use cargo::{core::PackageIdSpec, util::command_prelude::*};
use cargo_information::ops;

pub fn cli() -> Command {
    Command::new("cargo-info")
        .bin_name("cargo")
        .subcommand_required(true)
        .subcommand(info_subcommand())
}

fn info_subcommand() -> Command {
    Command::new("info")
        .about("Display info about a package in the registry")
        .arg(
            Arg::new("package")
                .required(true)
                .value_name("SPEC")
                .help_heading(heading::PACKAGE_SELECTION)
                .help("Package to inspect"),
        )
        .arg_index("Registry index URL to search packages in")
        .arg_registry("Registry to search packages in")
        .arg(
            opt(
                "verbose",
                "Use verbose output (-vv very verbose/build.rs output)",
            )
            .short('v')
            .action(ArgAction::Count)
            .global(true),
        )
        .arg(
            flag("quiet", "Do not print cargo log messages")
                .short('q')
                .global(true),
        )
        .arg(
            opt("color", "Coloring: auto, always, never")
                .value_name("WHEN")
                .global(true),
        )
        .arg(
            flag("frozen", "Require Cargo.lock and cache are up to date")
                .help_heading(heading::MANIFEST_OPTIONS)
                .global(true),
        )
        .arg(
            flag("locked", "Require Cargo.lock is up to date")
                .help_heading(heading::MANIFEST_OPTIONS)
                .global(true),
        )
        .arg(
            flag("offline", "Run without accessing the network")
                .help_heading(heading::MANIFEST_OPTIONS)
                .global(true),
        )
        .arg(multi_opt("config", "KEY=VALUE", "Override a configuration value").global(true))
        .arg(
            Arg::new("unstable-features")
                .help("Unstable (nightly-only) flags to Cargo, see 'cargo -Z help' for details")
                .short('Z')
                .value_name("FLAG")
                .action(ArgAction::Append)
                .global(true),
        )
        .after_help(color_print::cstr!(
            "Run `<cyan,bold>cargo help info</>` for more detailed information.\n"
        ))
}

pub fn exec(config: &mut Config, args: &ArgMatches) -> CliResult {
    let verbose = args.verbose();
    let quiet = args.flag("quiet");
    let color = args.get_one::<String>("color").cloned();
    let frozen = args.flag("frozen");
    let locked = args.flag("locked");
    let offline = args.flag("offline");
    let unstable_flags: Vec<String> = args
        .get_many::<String>("unstable-features")
        .unwrap_or_default()
        .cloned()
        .collect();
    let config_args: Vec<String> = args
        .get_many::<String>("config")
        .unwrap_or_default()
        .cloned()
        .collect();
    config.configure(
        verbose,
        quiet,
        color.as_deref(),
        frozen,
        locked,
        offline,
        &None,
        &unstable_flags,
        &config_args,
    )?;

    let package = args
        .get_one::<String>("package")
        .map(String::as_str)
        .unwrap();
    let spec = PackageIdSpec::parse(package)?;
    if spec.name().is_empty() {
        return Err(CliError::new(
            anyhow::format_err!("package ID specification must have a name"),
            101,
        ));
    }

    let reg_or_index = args.registry_or_index(config)?;
    ops::info(&spec, config, reg_or_index)?;
    Ok(())
}

#[test]
fn verify_cli() {
    cli().debug_assert();
}
