use cargo_test_macro::cargo_test;
use cargo_test_support::file;

use super::{cargo_info, init_registry_without_token};

#[cargo_test]
fn case() {
    init_registry_without_token();
    for ver in ["0.1.1+my-package", "0.2.0+my-package", "0.2.3+my-package"] {
        cargo_test_support::registry::Package::new("my-package", ver).publish();
    }
    cargo_info()
        .arg("my-package@0.2")
        .arg("--registry=dummy-registry")
        .assert()
        .success()
        .stdout_eq_(file!["stdout.log"])
        .stderr_eq_(file!["stderr.log"]);
}
