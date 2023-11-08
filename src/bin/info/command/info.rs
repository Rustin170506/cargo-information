use cargo::util::command_prelude::*;
use cargo_information::ops;

pub fn cli() -> Command {
    Command::new("info")
        .about("Display info about a package in the registry")
        .arg(Arg::new("pkgid").required(true).value_name("SPEC"))
        .arg(
            opt(
                "verbose",
                "Use verbose output (-vv very verbose/build.rs output)",
            )
            .short('v')
            .action(ArgAction::Count)
            .global(true),
        )
        .arg_quiet()
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

    let pkgid = args.get_one::<String>("pkgid").map(String::as_str).unwrap();
    ops::info(pkgid, config)?;
    Ok(())
}
