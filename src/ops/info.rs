use anstyle::{AnsiColor, Effects};
use cargo::util::style::{GOOD, NOP, WARN};
use cargo::{CargoResult, Config};
use crates_io_api::{FullCrate, SyncClient};
use std::time::Duration;

pub fn info(spec: &str, config: &Config) -> CargoResult<()> {
    let client = SyncClient::new(
        "cargo-information (github.com/hi-rustin/cargo-information)",
        Duration::from_millis(10),
    )?;

    let krate: FullCrate = client.full_crate(spec, false)?;

    pretty_view(&krate, config)?;

    Ok(())
}

fn pretty_view(krate: &FullCrate, config: &Config) -> CargoResult<()> {
    config.shell().write_stdout("\n", &NOP)?;
    config.shell().write_stdout(
        format!(
            "{name}@{version}",
            name = krate.name,
            version = krate.max_version
        ),
        &GOOD,
    )?;
    config.shell().write_stdout(" | ", &NOP)?;
    match krate.license {
        Some(ref license) => {
            config
                .shell()
                .write_stdout(format!("{license}", license = license), &GOOD)?;
        }
        None => {
            config.shell().write_stdout("No license", &WARN)?;
        }
    }
    config.shell().write_stdout(" | ", &NOP)?;
    config.shell().write_stdout("deps: ", &NOP)?;
    let deps = krate.reverse_dependencies.meta.total;
    config
        .shell()
        .write_stdout(deps, &AnsiColor::Cyan.on_default().effects(Effects::BOLD))?;
    config.shell().write_stdout(" | ", &NOP)?;
    config.shell().write_stdout("downloads: ", &NOP)?;

    config.shell().write_stdout(
        krate.total_downloads,
        &AnsiColor::Cyan.on_default().effects(Effects::BOLD),
    )?;
    config.shell().write_stdout("\n", &NOP)?;

    Ok(())
}
