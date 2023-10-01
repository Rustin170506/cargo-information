use cargo::core::Shell;

mod cli;
mod command;

fn main() {
    let result = cli::main();
    let mut shell = Shell::new();

    match result {
        Ok(()) => {}
        Err(e) => cargo::exit_with_error(e, &mut shell),
    }
}
