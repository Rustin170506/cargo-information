use cargo_test_macro::cargo_test;
use cargo_test_support::file;

use super::cargo_info_with_color;

#[cargo_test]
fn case() {
    cargo_info_with_color()
        .arg("--help")
        .arg("--registry=dummy-registry")
        .assert()
        .success()
        .stdout_matches(file!["stdout.log"])
        .stderr_matches(file!["stderr.log"]);
}
