use cargo_test_macro::cargo_test;
use snapbox::file;

use super::cargo_info;

#[cargo_test]
fn case() {
    cargo_info()
        .arg("--help")
        .arg("--registry=dummy-registry")
        .assert()
        .success()
        .stdout_matches(file!["stdout.log"])
        .stderr_matches(file!["stderr.log"]);
}
