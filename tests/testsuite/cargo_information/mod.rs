use cargo_test_support::{compare, TestEnv};

mod basic;
mod features;
mod help;
mod not_found;
mod with_frozen_outside_workspace;
mod with_frozen_within_workspace;
mod with_locked_outside_workspace;
mod with_locked_within_workspace;
mod with_offline;
mod with_quiet;
mod within_workspace;

// Invoke `cargo-info info` with the test environment.
pub(crate) fn cargo_info() -> snapbox::cmd::Command {
    snapbox::cmd::Command::new(snapbox::cmd::cargo_bin("cargo-info"))
        .with_assert(compare::assert_ui())
        .test_env()
        .arg("info")
        .arg("--color=never")
}
