use crate::command::info;
use cargo::util::command_prelude::*;

pub fn main(ctx: &mut GlobalContext) -> CliResult {
    let matches = info::cli().try_get_matches()?;
    match matches.subcommand() {
        Some(("info", args)) => info::exec(ctx, args),
        _ => unreachable!("clap should ensure we don't get here"),
    }
}
