use cargo_test_macro::cargo_test;
use cargo_test_support::compare;
use cargo_test_support::curr_dir;
use cargo_test_support::prelude::*;

#[cargo_test]
fn case() {
    snapbox::cmd::Command::new(snapbox::cmd::cargo_bin("cargo-info"))
        .with_assert(compare::assert_ui())
        .test_env()
        .arg("--help")
        .assert()
        .success()
        .stdout_matches_path(curr_dir!().join("stdout.log"))
        .stderr_matches_path(curr_dir!().join("stderr.log"));
}
