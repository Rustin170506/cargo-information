use crate::command::info;
use cargo::util::command_prelude::*;

pub fn main() -> CliResult {
    let args = info::cli().try_get_matches()?;
    info::exec(&args)
}
