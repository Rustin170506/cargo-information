use cargo_test_macro::cargo_test;
use cargo_test_support::{curr_dir, registry::RegistryBuilder};

use super::cargo_info;

#[cargo_test]
fn case() {
    let _ = RegistryBuilder::new()
        .alternative()
        .no_configure_token()
        .build();
    cargo_test_support::registry::Package::new("my-package", "99999.0.0-alpha.1+my-package")
        .alternative(true)
        .publish();

    cargo_info()
        .arg("https://crates.io")
        .arg("--registry=alternative")
        .assert()
        .failure()
        .stdout_matches_path(curr_dir!().join("stdout.log"))
        .stderr_matches_path(curr_dir!().join("stderr.log"));
}
