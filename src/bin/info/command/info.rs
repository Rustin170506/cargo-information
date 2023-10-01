use cargo::util::command_prelude::*;
use cargo_information::ops;

pub fn cli() -> Command {
    Command::new("info")
        .about("Display info about a package in the registry")
        .arg(Arg::new("pkgid").required(true).value_name("SPEC"))
        .arg_quiet()
        .after_help(color_print::cstr!(
            "Run `<cyan,bold>cargo help info</>` for more detailed information.\n"
        ))
}

pub fn exec(config: &mut Config, args: &ArgMatches) -> CliResult {
    let pkgid = args.get_one::<String>("pkgid").map(String::as_str).unwrap();

    ops::info(pkgid, config)?;
    Ok(())
}
