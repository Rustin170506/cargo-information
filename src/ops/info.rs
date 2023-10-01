use cargo::{CargoResult, Config};
use crates_io_api::{CrateResponse, SyncClient};
use std::time::Duration;

pub fn info(spec: &str, config: &Config) -> CargoResult<()> {
    let client = SyncClient::new(
        "cargo-information (github.com/hi-rustin/cargo-information)",
        Duration::from_millis(10),
    )?;

    let krate: CrateResponse = client.get_crate(spec)?;

    config
        .shell()
        .write_stdout(format!("{:?}", krate), &cargo::util::style::NOP)?;

    Ok(())
}
