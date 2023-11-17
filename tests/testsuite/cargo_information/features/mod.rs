use cargo_test_macro::cargo_test;
use cargo_test_support::{
    compare::{self},
    curr_dir, TestEnv,
};

#[cargo_test]
fn case() {
    cargo_test_support::registry::init();
    cargo_test_support::registry::Package::new("my-package", "0.1.1")
        .feature("default", &["feature1", "feature2"])
        .feature("feature1", &[])
        .feature("feature2", &[])
        .publish();

    snapbox::cmd::Command::new(snapbox::cmd::cargo_bin("info"))
        .with_assert(compare::assert_ui())
        .test_env()
        .arg("my-package")
        .arg("--color=never")
        .assert()
        .success()
        .stdout_matches_path(curr_dir!().join("stdout.log"))
        .stderr_matches_path(curr_dir!().join("stderr.log"));
}
