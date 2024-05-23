use crate::command::info;
use cargo::util::command_prelude::*;

pub fn main(gctx: &mut GlobalContext) -> CliResult {
    let matches = info::cli().try_get_matches()?;
    match matches.subcommand() {
        Some(("info", args)) => info::exec(gctx, args),
        _ => unreachable!("clap should ensure we don't get here"),
    }
}
