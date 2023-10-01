use crate::command::info;
use cargo::util::command_prelude::*;

pub fn main(config: &mut Config) -> CliResult {
    let args = info::cli().try_get_matches()?;
    info::exec(config, &args)
}
