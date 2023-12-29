use cargo_test_macro::cargo_test;
use cargo_test_support::curr_dir;

use super::{cargo_info, init_registry_without_token};

#[cargo_test]
fn case() {
    init_registry_without_token();
    cargo_test_support::registry::Package::new("my-package", "0.1.1+my-package")
        .rust_version("1.0.0")
        .publish();
    cargo_test_support::registry::Package::new("my-package", "0.2.0+my-package")
        .rust_version("1.9876.0")
        .publish();

    cargo_info()
        .arg("my-package")
        .arg("--registry=dummy-registry")
        .arg("--ignore-rust-version")
        .assert()
        .success()
        .stdout_matches_path(curr_dir!().join("stdout.log"))
        .stderr_matches_path(curr_dir!().join("stderr.log"));
}
