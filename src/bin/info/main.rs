use cargo::core::Shell;

mod cli;
mod command;

fn main() {
    let mut config = match cargo::Config::default() {
        Ok(cfg) => cfg,
        Err(e) => {
            let mut shell = Shell::new();
            cargo::exit_with_error(e.into(), &mut shell)
        }
    };
    let result = cli::main(&mut config);

    match result {
        Ok(()) => {}
        Err(e) => cargo::exit_with_error(e, &mut config.shell()),
    }
}
