use cargo::util::command_prelude::*;

pub fn cli() -> Command {
    Command::new("info")
        .about("Display info about a package in the registry")
        .arg(Arg::new("pkgid").required(true).value_name("SPEC"))
        .arg_index()
        .arg(opt("registry", "Registry to use").value_name("REGISTRY"))
        .arg_quiet()
        .after_help(color_print::cstr!(
            "Run `<cyan,bold>cargo help info</>` for more detailed information.\n"
        ))
}

pub fn exec(config: &mut Config, args: &ArgMatches) -> CliResult {
    let _registry = args.registry(config)?;
    let _index = args.index()?;
    let pkgid = args.get_one::<String>("pkgid").map(String::as_str).unwrap();

    println!("pkgid: {}", pkgid);

    Ok(())
}
