use cargo_test_macro::cargo_test;
use cargo_test_support::curr_dir;

use super::{cargo_info, init_registry_without_token};

#[cargo_test]
fn case() {
    init_registry_without_token();
    for ver in ["0.1.1+my-package", "0.2.0+my-package"] {
        cargo_test_support::registry::Package::new("my-package", ver).publish();
    }

    cargo_info()
        .arg("cargo-list-test-fixture")
        .arg("--path=./crate1")
        .current_dir(curr_dir!())
        .assert()
        .success()
        .stdout_matches_path(curr_dir!().join("stdout.log"))
        .stderr_matches_path(curr_dir!().join("stderr.log"));
}
