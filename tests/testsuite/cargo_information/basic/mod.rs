use cargo_test_macro::cargo_test;
use cargo_test_support::{
    compare::{self},
    curr_dir, TestEnv,
};

#[cargo_test]
fn case() {
    cargo_test_support::registry::init();
    for ver in [
        "0.1.1+my-package",
        "0.2.0+my-package",
        "0.2.3+my-package",
        "0.4.1+my-package",
        "20.0.0+my-package",
        "99999.0.0+my-package",
        "99999.0.0-alpha.1+my-package",
    ] {
        cargo_test_support::registry::Package::new("my-package", ver).publish();
    }

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
