use cargo::util::command_prelude::*;

pub fn cli() -> Command {
    Command::new("info")
        .about("Display info about a package in the registry")
        .arg(Arg::new("query").num_args(0..))
        .arg_index()
        .arg(opt("registry", "Registry to use").value_name("REGISTRY"))
        .arg_quiet()
        .after_help(color_print::cstr!(
            "Run `<cyan,bold>cargo help search</>` for more detailed information.\n"
        ))
}

pub fn exec(args: &ArgMatches) -> CliResult {
    Err(anyhow::format_err!("info: {:?}", args).into())
}
